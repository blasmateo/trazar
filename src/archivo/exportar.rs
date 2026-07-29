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
    
    // Resolver ruta del script Python: buscar el proyecto raíz caminando hacia arriba
    // desde el directorio del ejecutable hasta encontrar scripts/exportar-docx/
    let ruta_script = resolver_ruta_recurso(
        ruta_base,
        "scripts/exportar-docx/exportar_docx.py",
    );
    let ruta_script = match ruta_script {
        Some(p) => p,
        None => return Err(format!(
            "No se encontró la herramienta externa scripts/exportar-docx/exportar_docx.py (buscado desde {} hacia arriba)",
            ruta_base.display()
        )),
    };
    
    // Resolver intérprete de Python del .venv (mismo directorio raíz que el script)
    let raiz_proyecto = ruta_script
        .parent()              // .../scripts/exportar-docx/
        .and_then(|p| p.parent()) // .../scripts/
        .and_then(|p| p.parent()); // raíz del proyecto
    let python_venv = raiz_proyecto
        .map(|r| r.join(".venv/bin/python"))
        .unwrap_or_else(|| ruta_base.join(".venv/bin/python"));
    let python = if python_venv.exists() {
        python_venv
    } else {
        std::path::PathBuf::from("python3")
    };
    
    // Normalizar la ruta de salida a absoluta
    let ruta_salida_abs = if Path::new(ruta_salida).is_absolute() {
        std::path::PathBuf::from(ruta_salida)
    } else {
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")).join(ruta_salida)
    };
    
    // Invocar la herramienta externa
    let salida = std::process::Command::new(&python)
        .arg(&ruta_script)
        .arg("--json").arg(&ruta_json_final)
        .arg("--salida").arg(&ruta_salida_abs)
        .arg("--modo").arg(modo_str)
        .output()
        .map_err(|e| format!("Error al ejecutar herramienta externa: {}", e))?;
    
    // Mostrar salida del script
    if !salida.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&salida.stdout));
    }
    
    if !salida.status.success() {
        // El script imprime mensajes de error limpios en stderr; usarlos como
        // mensaje de error de trazar en vez de mostrar el traceback nativo.
        let msg = if !salida.stderr.is_empty() {
            String::from_utf8_lossy(&salida.stderr).trim().to_string()
        } else {
            format!("La herramienta externa falló con código {}", salida.status)
        };
        return Err(msg);
    }
    
    Ok(())
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