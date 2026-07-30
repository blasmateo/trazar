use std::fs;
use std::path::{Path, PathBuf};
use rustyline::DefaultEditor;

/// Remueve uno o varios cursos. Si no se especifican IDs, entra en modo interactivo.
pub fn ejecutar(ruta_base: &Path, nombres: Option<&[String]>) -> Result<(), String> {
    let ruta_cursos = ruta_base.join("datos/cursos");
    
    if !ruta_cursos.exists() {
        return Err("No existe el directorio de cursos".to_string());
    }
    
    let mut rl = match DefaultEditor::new() {
        Ok(editor) => editor,
        Err(e) => return Err(format!("Error al inicializar editor: {}", e)),
    };
    
    // Determinar qué cursos remover
    let cursos_existentes: Vec<PathBuf> = if let Some(lista) = nombres {
        if lista.is_empty() {
            return Err("Se requiere especificar al menos un curso a remover".to_string());
        }
        resolver_cursos(&ruta_cursos, lista)?
    } else {
        // Modo interactivo
        modo_interactivo(&ruta_cursos, &mut rl)?
    };
    
    if cursos_existentes.is_empty() {
        println!("No se seleccionó ningún curso. Operación cancelada.");
        return Ok(());
    }
    
    // Mostrar lo que se va a remover
    println!("\nSe removerán los siguientes cursos:");
    for ruta in &cursos_existentes {
        if let Some(nombre) = ruta.file_name() {
            println!("  - {}", nombre.to_string_lossy());
        }
    }
    
    // Confirmación
    match rl.readline("\nPara confirmar, ingrese 'Si': ") {
        Ok(confirmacion) => {
            if confirmacion.trim() != "Si" {
                println!("Operación cancelada. No se removió nada.");
                return Ok(());
            }
        }
        Err(_) => {
            println!("\nOperación cancelada. No se removió nada.");
            return Ok(());
        }
    }
    
    // Remover cursos
    for ruta in &cursos_existentes {
        fs::remove_dir_all(ruta)
            .map_err(|e| format!("Error al remover {}: {}", ruta.display(), e))?;
        
        if let Some(nombre) = ruta.file_name() {
            println!("Removido: {}", nombre.to_string_lossy());
        }
    }
    
    Ok(())
}

/// Resuelve una lista de nombres/IDs a rutas de cursos existentes.
fn resolver_cursos(ruta_cursos: &Path, nombres: &[String]) -> Result<Vec<PathBuf>, String> {
    let mut cursos_existentes: Vec<PathBuf> = Vec::new();
    let mut cursos_no_encontrados: Vec<String> = Vec::new();
    
    for nombre in nombres {
        let encontrado = buscar_curso(ruta_cursos, nombre)?;
        match encontrado {
            Some(ruta_completa) => cursos_existentes.push(ruta_completa),
            None => cursos_no_encontrados.push(nombre.clone()),
        }
    }
    
    if !cursos_no_encontrados.is_empty() {
        println!("Cursos no encontrados:");
        for nombre in &cursos_no_encontrados {
            println!("  - {}", nombre);
        }
    }
    
    Ok(cursos_existentes)
}

/// Modo interactivo: lista cursos y permite seleccionar varios.
fn modo_interactivo(ruta_cursos: &Path, rl: &mut DefaultEditor) -> Result<Vec<PathBuf>, String> {
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
    
    println!("Cursos registrados ({}):", cursos.len());
    for (id, nombre, _) in &cursos {
        println!("  [{}] {}", id, nombre);
    }
    
    let input = match rl.readline("\nIngrese los IDs a remover separados por espacio (o Enter para salir): ") {
        Ok(line) => line,
        Err(_) => return Ok(Vec::new()),
    };
    
    let input = input.trim();
    if input.is_empty() {
        return Ok(Vec::new());
    }
    
    let mut seleccionados: Vec<PathBuf> = Vec::new();
    for id_str in input.split_whitespace() {
        if let Ok(id) = id_str.parse::<u32>() {
            if let Some((_, _, ruta)) = cursos.iter().find(|(curso_id, _, _)| *curso_id == id) {
                seleccionados.push(ruta.clone());
            } else {
                println!("⚠ ID {} no encontrado, se omite.", id);
            }
        } else {
            println!("⚠ '{}' no es un número válido, se omite.", id_str);
        }
    }
    
    Ok(seleccionados)
}

/// Busca un curso por nombre o ID (prefijo numérico).
fn buscar_curso(ruta_cursos: &Path, nombre: &str) -> Result<Option<PathBuf>, String> {
    let entradas = fs::read_dir(ruta_cursos)
        .map_err(|e| format!("Error al leer directorio: {}", e))?;
    
    let nombre_lower = nombre.to_lowercase();
    let mut candidato_parcial: Option<PathBuf> = None;
    
    for entrada in entradas {
        let entrada = entrada.map_err(|e| format!("Error al leer entrada: {}", e))?;
        let ruta = entrada.path();
        
        if ruta.is_dir() {
            let nombre_dir = entrada.file_name();
            let nombre_str = nombre_dir.to_string_lossy();
            
            // Coincidencia exacta
            if nombre_str == nombre {
                return Ok(Some(ruta));
            }
            
            // Coincidencia por ID numérico
            if let Some(id_str) = nombre_str.split('-').next() {
                if id_str == nombre {
                    return Ok(Some(ruta));
                }
            }
            
            // Coincidencia por nombre simple (después del ID)
            if let Some(simple) = nombre_str.splitn(2, '-').nth(1) {
                if simple == nombre_lower {
                    candidato_parcial = Some(ruta);
                }
            }
        }
    }
    
    // Si no encontramos coincidencia exacta pero sí parcial, devolver el parcial
    if let Some(ruta) = candidato_parcial {
        return Ok(Some(ruta));
    }
    
    Ok(None)
}