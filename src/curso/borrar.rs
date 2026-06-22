use std::fs;
use std::path::{Path, PathBuf};
use rustyline::DefaultEditor;

/// Elimina uno o varios cursos especificados por nombre o ID.
pub fn ejecutar(ruta_base: &Path, nombres: &[String]) -> Result<(), String> {
    if nombres.is_empty() {
        return Err("Debe especificar al menos un curso a borrar".to_string());
    }
    
    let ruta_cursos = ruta_base.join("datos/cursos");
    
    if !ruta_cursos.exists() {
        return Err("No existe el directorio de cursos".to_string());
    }
    
    // Verificar qué cursos existen
    let mut cursos_existentes: Vec<PathBuf> = Vec::new();
    let mut cursos_no_encontrados: Vec<String> = Vec::new();
    
    for nombre in nombres {
        let encontrado = buscar_curso(&ruta_cursos, nombre)?;
        match encontrado {
            Some(ruta_completa) => cursos_existentes.push(ruta_completa),
            None => cursos_no_encontrados.push(nombre.clone()),
        }
    }
    
    if !cursos_no_encontrados.is_empty() {
        println!("!! Cursos no encontrados:");
        for nombre in &cursos_no_encontrados {
            println!("  - {}", nombre);
        }
    }
    
    if cursos_existentes.is_empty() {
        return Err("No se encontró ningún curso válido para borrar".to_string());
    }
    
    // Confirmación
    println!("\nSe borrarán los siguientes cursos:");
    for ruta in &cursos_existentes {
        if let Some(nombre) = ruta.file_name() {
            println!("  - {}", nombre.to_string_lossy());
        }
    }
    
	// Confirmación
	let mut rl = match DefaultEditor::new() {
		Ok(editor) => editor,
		Err(e) => return Err(format!("Error al inicializar editor: {}", e)),
	};

	match rl.readline("\nPara confirmar, escriba 'borrar-curso': ") {
		Ok(confirmacion) => {
			if confirmacion.trim() != "borrar-curso" {
				println!("Operación cancelada. No se borró nada.");
				return Ok(());
			}
		}
		Err(_) => {
			println!("\nOperación cancelada. No se borró nada.");
			return Ok(());
		}
	}
    
    // Borrar cursos
    for ruta in &cursos_existentes {
        fs::remove_dir_all(ruta)
            .map_err(|e| format!("Error al borrar {}: {}", ruta.display(), e))?;
        
        if let Some(nombre) = ruta.file_name() {
            println!("Borrado: {}", nombre.to_string_lossy());
        }
    }
    
    Ok(())
}

/// Busca un curso por nombre o ID (prefijo numérico).
fn buscar_curso(ruta_cursos: &Path, nombre: &str) -> Result<Option<PathBuf>, String> {
    let entradas = fs::read_dir(ruta_cursos)
        .map_err(|e| format!("Error al leer directorio: {}", e))?;
    
    for entrada in entradas {
        let entrada = entrada.map_err(|e| format!("Error al leer entrada: {}", e))?;
        let ruta = entrada.path();
        
        if ruta.is_dir() {
            let nombre_dir = entrada.file_name();
            let nombre_str = nombre_dir.to_string_lossy();
            
            // Coincidencia exacta por nombre completo
            if nombre_str == nombre {
                return Ok(Some(ruta));
            }
            
            // Coincidencia por ID (prefijo numérico)
            if let Some(id_str) = nombre_str.split('-').next() {
                if id_str == nombre {
                    return Ok(Some(ruta));
                }
            }
        }
    }
    
    Ok(None)
}