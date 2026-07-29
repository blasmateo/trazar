use std::path::Path;

/// Exporta datos de métricas a .docx invocando la herramienta externa en Python
pub fn ejecutar(ruta_base: &Path, tipo_str: &str, modo_str: &str, ruta_salida: &str) -> Result<(), String> {
    if tipo_str != "asistencias" {
        return Err(format!("Exportación de '{}' aún no implementada. Solo 'asistencias' está disponible.", tipo_str));
    }
    
    let ruta_cursos = ruta_base.join("datos/cursos");
    
    if !ruta_cursos.exists() {
        return Err("No existe el directorio datos/cursos/".to_string());
    }
    
    // Obtener lista de cursos
    let cursos = obtener_cursos(&ruta_cursos)?;
    
    if cursos.is_empty() {
        return Err("No hay cursos registrados.".to_string());
    }
    
    // Seleccionar curso
    let nombre_curso = if cursos.len() == 1 {
        cursos[0].clone()
    } else {
        seleccionar_curso(&cursos)?
    };
    
    // Buscar archivo de métricas según modo
    let ruta_metricas = ruta_cursos.join(&nombre_curso).join("metricas");
    
    if !ruta_metricas.exists() {
        return Err(format!(
            "No hay métricas guardadas para el curso '{}'.\nUse 'trazar metricas calcular -t asistencias -a' para generar métricas.",
            nombre_curso
        ));
    }
    
    let archivo_json = match modo_str {
        "tabla" => "asistencias-tabla.json",
        _ => "asistencias-resumen.json",
    };
    
    let ruta_json = ruta_metricas.join(archivo_json);
    
    // Fallback al otro archivo si el específico no existe
    let ruta_json_final = if !ruta_json.exists() {
        let fallback = if modo_str == "tabla" {
            ruta_metricas.join("asistencias-resumen.json")
        } else {
            ruta_metricas.join("asistencias-tabla.json")
        };
        
        if fallback.exists() {
            eprintln!("ℹ Archivo '{}' no encontrado. Usando '{}' en su lugar.",
                     archivo_json,
                     fallback.file_name().unwrap_or_default().to_string_lossy());
            fallback
        } else {
            return Err(format!(
                "No se encontró archivo de métricas para el curso '{}'.\nUse 'trazar metricas calcular -t asistencias -a' para generar métricas.",
                nombre_curso
            ));
        }
    } else {
        ruta_json
    };
    
    // Resolver la herramienta externa de exportación.
    //
    // Convención de DISTRIBUCIÓN (binario PyInstaller junto al ejecutable Rust):
    //   <dir-ejecutable>/trazar
    //   <dir-ejecutable>/_scripts/exportar-docx       # binario autocontenido
    //
    //   El binario empaqueta el intérprete Python y python-docx, por lo que NO
    //   requiere Python instalado en la máquina destino.
    //
    // Convención de DESARROLLO (fallback):
    //   <raíz-proyecto>/scripts/exportar-docx/exportar_docx.py
    //   <raíz-proyecto>/.venv/bin/python
    //
    // Se busca primero el binario de distribución; si no existe, modo dev.
    let dir_exe = ruta_base;

    // Normalizar la ruta de salida a absoluta
    let ruta_salida_abs = if Path::new(ruta_salida).is_absolute() {
        std::path::PathBuf::from(ruta_salida)
    } else {
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")).join(ruta_salida)
    };

    // Leer el JSON de métricas para embeberlo en el envelope del contrato
    let datos_json = std::fs::read_to_string(&ruta_json_final)
        .map_err(|e| format!("Error al leer métricas '{}': {}", ruta_json_final.display(), e))?;
    let datos_valor: serde_json::Value = serde_json::from_str(&datos_json)
        .map_err(|e| format!("Métricas '{}' no es JSON válido: {}", ruta_json_final.display(), e))?;

    // Envelope del contrato IPC (se envía por stdin)
    let envelope = serde_json::json!({
        "contractVersion": "1.0",
        "operation": "exportar-docx",
        "payload": { "datos": datos_valor },
        "output": {
            "ruta": ruta_salida_abs.to_string_lossy(),
            "modo": modo_str,
        }
    });
    let envelope_str = serde_json::to_string(&envelope)
        .map_err(|e| format!("Error al serializar envelope: {}", e))?;

    // --- 1) Distribución: buscar binario PyInstaller ---
    //
    // Se busca en este orden:
    //   a) <dir_exe>/_scripts/exportar-docx          — junto al ejecutable (distribución real)
    //   b) target/release/_scripts/exportar-docx     — build.sh por defecto
    //   c) target/debug/_scripts/exportar-docx       — build.sh target/debug
    //
    // Esto cubre cualquier combinación de perfil de compilación (debug/release)
    // contra el destino con el que se haya ejecutado build.sh.
    let exe_dist = dir_exe.join("_scripts/exportar-docx");
    #[cfg(windows)]
    let exe_dist = dir_exe.join("_scripts/exportar-docx.exe");

    if exe_dist.exists() {
        let resultado = std::process::Command::new(&exe_dist)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();
        return ejecutar_contrato(resultado, envelope_str);
    }

    // Fallback: buscar en target/<profile>/_scripts/ caminando hacia arriba
    // desde el directorio del ejecutable. Esto permite que build.sh (que por
    // defecto instala en target/release) funcione aunque el binario Rust se
    // haya compilado en debug (y viceversa).
    let en_target = resolver_ruta_recurso(dir_exe, "target/release/_scripts/exportar-docx")
        .or_else(|| resolver_ruta_recurso(dir_exe, "target/debug/_scripts/exportar-docx"));

    if let Some(exe_target) = en_target {
        let resultado = std::process::Command::new(&exe_target)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();
        return ejecutar_contrato(resultado, envelope_str);
    }

    // --- 2) Desarrollo: scripts/exportar-docx/exportar_docx.py + .venv/bin/python ---
    let script_dev = resolver_ruta_recurso(
        dir_exe,
        "scripts/exportar-docx/exportar_docx.py",
    );
    let script_dev = match script_dev {
        Some(p) => p,
        None => return Err(format!(
            "No se encontró la herramienta externa de exportación.\n\
             En distribución:  {}/_scripts/exportar-docx (genérelo con scripts/exportar-docx/build.sh)\n\
             En desarrollo:    scripts/exportar-docx/exportar_docx.py (desde {} hacia arriba)",
            dir_exe.display(), dir_exe.display()
        )),
    };
    let raiz_proyecto = script_dev
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent());
    let python = raiz_proyecto
        .map(|r| r.join(".venv/bin/python"))
        .filter(|p| p.exists())
        .unwrap_or_else(|| std::path::PathBuf::from("python3"));

    let resultado = std::process::Command::new(&python)
        .arg(&script_dev)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();

    ejecutar_contrato(resultado, envelope_str)
}

/// Ejecuta la comunicación con la herramienta externa según el contrato IPC:
/// escribe el envelope JSON por stdin del hijo, lee stdout (JSON de resultado)
/// y stderr (logs humanos), y mapea el código de salida a un mensaje accionable.
fn ejecutar_contrato(
    spawn: std::io::Result<std::process::Child>,
    envelope: String,
) -> Result<(), String> {
    let mut child = spawn.map_err(|e| format!("Error al iniciar herramienta externa: {}", e))?;

    // Escribir el envelope por stdin
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(envelope.as_bytes())
            .map_err(|e| format!("Error al enviar datos a la herramienta: {}", e))?;
        // drop stdin para señalizar EOF
    }

    let output = child.wait_with_output()
        .map_err(|e| format!("Error al esperar la herramienta externa: {}", e))?;

    // stderr: logs humanos ([INFO]/[WARN]/[ERR]) — mostrar solo esos al usuario
    if !output.stderr.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        for linea in stderr.lines() {
            if linea.starts_with("[INFO]") || linea.starts_with("[WARN]") || linea.starts_with("[ERR]") {
                eprintln!("{}", linea);
            }
        }
    }

    let codigo = output.status.code().unwrap_or(-1);
    let stdout_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if codigo == 0 {
        // Éxito: parsear artefactos del JSON de stdout para confirmar
        if let Ok(resp) = serde_json::from_str::<serde_json::Value>(&stdout_str) {
            if resp.get("status").and_then(|s| s.as_str()) == Some("ok") {
                if let Some(arts) = resp.get("artefactos").and_then(|a| a.as_array()) {
                    if let Some(primero) = arts.first() {
                        if let Some(ruta) = primero.get("ruta").and_then(|r| r.as_str()) {
                            println!("✓ Documento exportado: {}", ruta);
                            return Ok(());
                        }
                    }
                }
            }
        }
        if !stdout_str.is_empty() {
            println!("{}", stdout_str);
        }
        return Ok(());
    }
    _procesar_error(codigo, stdout_str)
}

/// Mapea el código de salida semántico de la herramienta externa a un mensaje
/// accionable para el usuario. Intenta extraer el detalle del JSON de error.
fn _procesar_error(codigo: i32, stdout_str: String) -> Result<(), String> {
    let msg_error = if let Ok(resp) = serde_json::from_str::<serde_json::Value>(&stdout_str) {
        if resp.get("status").and_then(|s| s.as_str()) == Some("error") {
            if let Some(err) = resp.get("error") {
                if let Some(m) = err.get("mensaje").and_then(|m| m.as_str()) {
                    return Err(m.to_string());
                }
            }
        }
        stdout_str
    } else {
        String::new()
    };

    let msg = match codigo {
        1 => format!("{}\n  La herramienta externa no soporta la versión del contrato. Actualice el binario con scripts/exportar-docx/build.sh.", msg_error),
        2 => if msg_error.is_empty() { "Datos de entrada inválidos. Verifique que existan métricas calculadas para este curso.".to_string() } else { msg_error },
        3 => if msg_error.is_empty() { "No se pudo crear el documento. Verifique que la ruta sea un archivo .docx válido y tenga permisos de escritura.".to_string() } else { msg_error },
        4 => format!("{}\n  Falta una dependencia en la herramienta externa. Regénere el binario con scripts/exportar-docx/build.sh.", msg_error),
        _ => if msg_error.is_empty() { format!("La herramienta externa falló (código {}).", codigo) } else { msg_error },
    };
    Err(msg)
}





/// Busca un recurso relativo caminando hacia arriba desde `inicio` (típico: el
/// directorio del ejecutable) hasta la raíz del sistema de archivos. Devuelve
/// la primera ruta donde exista `recurso_relativo`.
fn resolver_ruta_recurso(inicio: &Path, recurso_relativo: &str) -> Option<std::path::PathBuf> {
    let mut actual = inicio.to_path_buf();
    loop {
        let candidata = actual.join(recurso_relativo);
        if candidata.exists() {
            return Some(candidata);
        }
        if !actual.pop() {
            break;
        }
    }
    None
}

/// Obtiene la lista de cursos desde datos/cursos/
fn obtener_cursos(ruta_cursos: &Path) -> Result<Vec<String>, String> {
    let mut cursos: Vec<String> = Vec::new();
    
    let entradas = std::fs::read_dir(ruta_cursos)
        .map_err(|e| format!("Error al leer directorio de cursos: {}", e))?;
    
    for entrada in entradas {
        let entrada = entrada.map_err(|e| format!("Error al leer entrada: {}", e))?;
        let ruta = entrada.path();
        
        if ruta.is_dir() {
            if let Some(nombre) = ruta.file_name() {
                let nombre_str = nombre.to_string_lossy().to_string();
                if let Some(id_str) = nombre_str.split('-').next() {
                    if id_str.parse::<u32>().is_ok() {
                        cursos.push(nombre_str);
                    }
                }
            }
        }
    }
    
    cursos.sort();
    Ok(cursos)
}

/// Selecciona un curso de forma interactiva
fn seleccionar_curso(cursos: &[String]) -> Result<String, String> {
    println!("Cursos disponibles:");
    for (i, curso) in cursos.iter().enumerate() {
        println!("  {}) {}", i + 1, curso);
    }
    
    print!("\nSeleccione un curso (número): ");
    use std::io::Write;
    std::io::stdout().flush().map_err(|e| format!("Error: {}", e))?;
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)
        .map_err(|e| format!("Error al leer entrada: {}", e))?;
    
    let input = input.trim();
    let num: usize = input.parse()
        .map_err(|_| format!("Entrada no válida: '{}'", input))?;
    
    if num == 0 || num > cursos.len() {
        return Err(format!("Número fuera de rango: {}", num));
    }
    
    Ok(cursos[num - 1].clone())
}