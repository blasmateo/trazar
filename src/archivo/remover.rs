use std::fs;
use std::path::{Path, PathBuf};
use rustyline::DefaultEditor;

/// Remueve uno o varios archivos. Si no se especifican rutas, entra en modo interactivo.
pub fn ejecutar(ruta_base: &Path, rutas: Option<&[String]>) -> Result<(), String> {
    let ruta_cursos = ruta_base.join("datos/cursos");
    
    if !ruta_cursos.exists() {
        return Err("No existe el directorio de cursos".to_string());
    }
    
    let mut rl = match DefaultEditor::new() {
        Ok(editor) => editor,
        Err(e) => return Err(format!("Error al inicializar editor: {}", e)),
    };
    
    // Determinar qué archivos remover
    let archivos_existentes: Vec<PathBuf> = if let Some(lista) = rutas {
        if lista.is_empty() {
            return Err("Se requiere especificar al menos un archivo a remover".to_string());
        }
        resolver_archivos(ruta_base, lista)?
    } else {
        // Modo interactivo
        modo_interactivo(ruta_base, &ruta_cursos, &mut rl)?
    };
    
    if archivos_existentes.is_empty() {
        println!("No se seleccionó ningún archivo. Operación cancelada.");
        return Ok(());
    }
    
    // Mostrar lo que se va a remover
    println!("\nSe removerán los siguientes archivos:");
    for ruta in &archivos_existentes {
        // Mostrar la ruta relativa desde datos/cursos/
        let ruta_relativa = ruta.strip_prefix(&ruta_base)
            .unwrap_or(ruta);
        println!("  - {}", ruta_relativa.to_string_lossy());
    }
    
    // Confirmación
    match rl.readline("\nPara confirmar, ingrese 'remover-archivo': ") {
        Ok(confirmacion) => {
            if confirmacion.trim() != "remover-archivo" {
                println!("Operación cancelada. No se removió nada.");
                return Ok(());
            }
        }
        Err(_) => {
            println!("\nOperación cancelada. No se removió nada.");
            return Ok(());
        }
    }
    
    // Remover archivos
    for ruta in &archivos_existentes {
        if ruta.is_dir() {
            fs::remove_dir_all(ruta)
                .map_err(|e| format!("Error al remover {}: {}", ruta.display(), e))?;
        } else {
            fs::remove_file(ruta)
                .map_err(|e| format!("Error al remover {}: {}", ruta.display(), e))?;
        }
        
        if let Some(nombre) = ruta.file_name() {
            println!("Removido: {}", nombre.to_string_lossy());
        }
    }
    
    Ok(())
}

/// Resuelve una lista de rutas a rutas de archivos existentes.
fn resolver_archivos(ruta_base: &Path, rutas: &[String]) -> Result<Vec<PathBuf>, String> {
    let mut archivos_existentes: Vec<PathBuf> = Vec::new();
    let mut archivos_no_encontrados: Vec<String> = Vec::new();
    
    for ruta_str in rutas {
        let ruta = Path::new(ruta_str);
        
        if ruta.exists() {
            archivos_existentes.push(ruta.to_path_buf());
        } else {
            // Intentar buscar en datos/cursos/*/archivo/
            let encontrado = buscar_archivo_en_cursos(ruta_base, ruta_str)?;
            match encontrado {
                Some(ruta_completa) => archivos_existentes.push(ruta_completa),
                None => archivos_no_encontrados.push(ruta_str.clone()),
            }
        }
    }
    
    if !archivos_no_encontrados.is_empty() {
        println!("Archivos no encontrados:");
        for nombre in &archivos_no_encontrados {
            println!("  - {}", nombre);
        }
    }
    
    Ok(archivos_existentes)
}

/// Busca un archivo por nombre en todos los cursos.
fn buscar_archivo_en_cursos(ruta_base: &Path, nombre: &str) -> Result<Option<PathBuf>, String> {
    let ruta_cursos = ruta_base.join("datos/cursos");
    
    let entradas = fs::read_dir(&ruta_cursos)
        .map_err(|e| format!("Error al leer directorio de cursos: {}", e))?;
    
    for entrada in entradas {
        let entrada = entrada.map_err(|e| format!("Error al leer entrada: {}", e))?;
        let ruta_curso = entrada.path();
        
        if ruta_curso.is_dir() {
            // Buscar en archivo/asistencias/
            let ruta_asistencias = ruta_curso.join("archivo/asistencias");
            if let Ok(encontrado) = buscar_archivo_recursivo(&ruta_asistencias, nombre) {
                if encontrado.is_some() {
                    return Ok(encontrado);
                }
            }
        }
    }
    
    Ok(None)
}

/// Busca archivo recursivamente en un directorio.
fn buscar_archivo_recursivo(ruta: &Path, nombre: &str) -> Result<Option<PathBuf>, String> {
    if !ruta.exists() {
        return Ok(None);
    }
    
    let entradas = fs::read_dir(ruta)
        .map_err(|e| format!("Error al leer directorio {}: {}", ruta.display(), e))?;
    
    for entrada in entradas {
        let entrada = entrada.map_err(|e| format!("Error al leer entrada: {}", e))?;
        let ruta_item = entrada.path();
        
        if ruta_item.is_dir() {
            if let Ok(encontrado) = buscar_archivo_recursivo(&ruta_item, nombre) {
                if encontrado.is_some() {
                    return Ok(encontrado);
                }
            }
        } else if ruta_item.is_file() {
            if let Some(nombre_archivo) = ruta_item.file_name() {
                if nombre_archivo == nombre {
                    return Ok(Some(ruta_item));
                }
            }
        }
    }
    
    Ok(None)
}

/// Modo interactivo: lista cursos y permite seleccionar archivos.
fn modo_interactivo(_ruta_base: &Path, ruta_cursos: &Path, rl: &mut DefaultEditor) -> Result<Vec<PathBuf>, String> {
    // Lista cursos
    let mut cursos: Vec<(u32, String, PathBuf)> = Vec::new();
    let entradas = fs::read_dir(ruta_cursos)
        .map_err(|e| format!("Error al leer directorio de cursos: {}", e))?;
    
    for entrada in entradas {
        let entrada = entrada.map_err(|e| format!("Error al leer entrada: {}", e))?;
        let ruta = entrada.path();
        
        if ruta.is_dir() {
            if let Some(nombre) = ruta.file_name() {
                let nombre_str = nombre.to_string_lossy().to_string();
                if let Some(id_str) = nombre_str.split('-').next() {
                    if let Ok(id) = id_str.parse::<u32>() {
                        cursos.push((id, nombre_str, ruta));
                    }
                }
            }
        }
    }
    
    if cursos.is_empty() {
        println!("ℹ No hay cursos registrados.");
        return Ok(Vec::new());
    }
    
    cursos.sort_by_key(|(id, _, _)| *id);
    
    // Seleccionar curso
    println!("Cursos con archivos:");
    for (id, nombre, _) in &cursos {
        println!("  [{}] {}", id, nombre);
    }
    
    let input = match rl.readline("\nSeleccione un curso por ID (o Enter para salir): ") {
        Ok(line) => line,
        Err(_) => return Ok(Vec::new()),
    };
    
    let input = input.trim();
    if input.is_empty() {
        return Ok(Vec::new());
    }
    
    let id_seleccionado: u32 = input.parse()
        .map_err(|_| "Se requiere ingresar un número válido".to_string())?;
    
    let ruta_curso = match cursos.iter().find(|(curso_id, _, _)| *curso_id == id_seleccionado) {
        Some((_, _, ruta)) => ruta.clone(),
        None => {
            println!("⚠ ID {} no encontrado.", id_seleccionado);
            return Ok(Vec::new());
        }
    };
    
    // Listar archivos del curso
    let ruta_archivo = ruta_curso.join("archivo/asistencias");
    if !ruta_archivo.exists() {
        println!("ℹ No hay archivos importados en este curso.");
        return Ok(Vec::new());
    }
    
    // Recopilar todos los archivos
    let mut archivos: Vec<(String, PathBuf)> = Vec::new();
    recopilar_archivos(&ruta_archivo, &ruta_archivo, &mut archivos)?;
    
    if archivos.is_empty() {
        println!("ℹ No hay archivos importados en este curso.");
        return Ok(Vec::new());
    }
    
    // Mostrar archivos con numeración
    println!("\nArchivos disponibles ({}):", archivos.len());
    for (nombre, _) in &archivos {
        println!("  - {}", nombre);
    }
    
    // Seleccionar archivos
    let input = match rl.readline("\nIngrese los nombres de archivos a remover separados por espacio (o Enter para salir): ") {
        Ok(line) => line,
        Err(_) => return Ok(Vec::new()),
    };
    
    let input = input.trim();
    if input.is_empty() {
        return Ok(Vec::new());
    }
    
    let mut seleccionados: Vec<PathBuf> = Vec::new();
    for nombre_archivo in input.split_whitespace() {
        if let Some((_, ruta_completa)) = archivos.iter().find(|(nombre, _)| nombre == nombre_archivo) {
            seleccionados.push(ruta_completa.clone());
        } else {
            println!("⚠ Archivo '{}' no encontrado, se omite.", nombre_archivo);
        }
    }
    
    Ok(seleccionados)
}

/// Recopila todos los archivos en un directorio recursivamente, guardando la ruta relativa a la asignatura.
fn recopilar_archivos(ruta: &Path, ruta_base: &Path, archivos: &mut Vec<(String, PathBuf)>) -> Result<(), String> {
    let entradas = fs::read_dir(ruta)
        .map_err(|e| format!("Error al leer directorio {}: {}", ruta.display(), e))?;
    
    for entrada in entradas {
        let entrada = entrada.map_err(|e| format!("Error al leer entrada: {}", e))?;
        let ruta_item = entrada.path();
        
        if ruta_item.is_dir() {
            recopilar_archivos(&ruta_item, ruta_base, archivos)?;
        } else if ruta_item.is_file() {
            // Calcular la ruta relativa desde archivo/asistencias/
            let ruta_relativa = ruta_item.strip_prefix(ruta_base)
                .unwrap_or(&ruta_item);
            let nombre_display = ruta_relativa.to_string_lossy().to_string();
            archivos.push((nombre_display, ruta_item));
        }
    }
    
    Ok(())
}
