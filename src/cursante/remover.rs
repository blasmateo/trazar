use std::fs;
use std::path::{Path, PathBuf};
use rustyline::DefaultEditor;
use crate::curso::preguntas::*;

/// Remueve uno o varios cursantes de un curso. Si no se especifican IDs, modo interactivo.
pub fn ejecutar(ruta_base: &Path, curso_arg: Option<&str>, nombres: Option<&[String]>) -> Result<(), String> {
    let ruta_cursos = ruta_base.join("datos/cursos");
    
    if !ruta_cursos.exists() {
        return Err("No existe el directorio de cursos".to_string());
    }
    
    let mut cursos: Vec<(u32, String)> = Vec::new();
    let entradas = fs::read_dir(&ruta_cursos)
        .map_err(|e| format!("Error al leer directorio de cursos: {}", e))?;
    
    for entrada in entradas {
        let entrada = entrada.map_err(|e| format!("Error al leer entrada: {}", e))?;
        let ruta = entrada.path();
        
        if ruta.is_dir() {
            if let Some(nombre) = ruta.file_name() {
                let nombre_str = nombre.to_string_lossy().to_string();
                if let Some(id_str) = nombre_str.split('-').next() {
                    if let Ok(id) = id_str.parse::<u32>() {
                        cursos.push((id, nombre_str));
                    }
                }
            }
        }
    }
    
    if cursos.is_empty() {
        return Err("No hay cursos registrados".to_string());
    }
    
    cursos.sort_by_key(|(id, _)| *id);
    
    let mut rl = match DefaultEditor::new() {
        Ok(editor) => editor,
        Err(e) => return Err(format!("Error al inicializar editor: {}", e)),
    };
    
    let nombre_curso = if let Some(arg) = curso_arg {
        encontrar_curso(&cursos, arg)?
    } else {
        println!("Cursos disponibles:");
        for (id, nombre) in &cursos {
            println!("  [{}] {}", id, nombre);
        }
        
        let input = match rl.readline("\nSeleccione un curso por ID (o Enter para salir): ") {
            Ok(line) => line,
            Err(_) => return Ok(()),
        };
        
        let input = input.trim();
        if input.is_empty() {
            return Ok(());
        }
        
        let id_seleccionado: u32 = input.parse()
            .map_err(|_| "Se requiere ingresar un número válido".to_string())?;
        
        encontrar_curso(&cursos, &id_seleccionado.to_string())?
    };
    
    let ruta_curso = ruta_cursos.join(&nombre_curso);
    let ruta_cursantes = ruta_curso.join("cursantes");
    
    if !ruta_cursantes.exists() {
        return Err(format!("No hay cursantes registrados en '{}'", nombre_curso));
    }
    
    // Determinar qué cursantes remover
    let cursantes_existentes: Vec<PathBuf> = if let Some(lista) = nombres {
        if lista.is_empty() {
            return Err("Se requiere especificar al menos un cursante a remover".to_string());
        }
        resolver_cursantes(&ruta_cursantes, lista)?
    } else {
        // Modo interactivo
        modo_interactivo(&ruta_cursantes, &mut rl)?
    };
    
    if cursantes_existentes.is_empty() {
        println!("No se seleccionó ningún cursante. Operación cancelada.");
        return Ok(());
    }
    
    println!("\nSe removerán los siguientes cursantes de '{}':", nombre_curso);
    for ruta in &cursantes_existentes {
        if let Some(nombre) = ruta.file_name() {
            println!("  - {}", nombre.to_string_lossy());
        }
    }
    
    match rl.readline("\nPara confirmar, ingrese 'remover-cursante': ") {
        Ok(confirmacion) => {
            if confirmacion.trim() != "remover-cursante" {
                println!("Operación cancelada. No se removió nada.");
                return Ok(());
            }
        }
        Err(_) => {
            println!("\nOperación cancelada. No se removió nada.");
            return Ok(());
        }
    }
    
    for ruta in &cursantes_existentes {
        fs::remove_dir_all(ruta)
            .map_err(|e| format!("Error al remover {}: {}", ruta.display(), e))?;
        
        if let Some(nombre) = ruta.file_name() {
            println!("Removido: {}", nombre.to_string_lossy());
        }
    }
    
    Ok(())
}

/// Resuelve una lista de nombres/IDs a rutas de cursantes existentes.
fn resolver_cursantes(ruta_cursantes: &Path, nombres: &[String]) -> Result<Vec<PathBuf>, String> {
    let mut cursantes_existentes: Vec<PathBuf> = Vec::new();
    let mut cursantes_no_encontrados: Vec<String> = Vec::new();
    
    for nombre in nombres {
        let encontrado = buscar_cursante(ruta_cursantes, nombre)?;
        match encontrado {
            Some(ruta_completa) => cursantes_existentes.push(ruta_completa),
            None => cursantes_no_encontrados.push(nombre.clone()),
        }
    }
    
    if !cursantes_no_encontrados.is_empty() {
        println!("Cursantes no encontrados:");
        for nombre in &cursantes_no_encontrados {
            println!("  - {}", nombre);
        }
    }
    
    Ok(cursantes_existentes)
}

/// Modo interactivo: lista cursantes y permite seleccionar varios.
fn modo_interactivo(ruta_cursantes: &Path, rl: &mut DefaultEditor) -> Result<Vec<PathBuf>, String> {
    let mut cursantes: Vec<(u32, String, PathBuf)> = Vec::new();
    
    let entradas = fs::read_dir(ruta_cursantes)
        .map_err(|e| format!("Error al leer directorio de cursantes: {}", e))?;
    
    for entrada in entradas {
        let entrada = entrada.map_err(|e| format!("Error al leer entrada: {}", e))?;
        let ruta = entrada.path();
        
        if ruta.is_dir() {
            if let Some(nombre) = ruta.file_name() {
                let nombre_str = nombre.to_string_lossy().to_string();
                if let Some(id_str) = nombre_str.split('-').next() {
                    if let Ok(id) = id_str.parse::<u32>() {
                        cursantes.push((id, nombre_str, ruta));
                    }
                }
            }
        }
    }
    
    if cursantes.is_empty() {
        println!("ℹ No hay cursantes registrados.");
        return Ok(Vec::new());
    }
    
    cursantes.sort_by_key(|(id, _, _)| *id);
    
    println!("Cursantes registrados ({}):", cursantes.len());
    for (id, nombre, _) in &cursantes {
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
            if let Some((_, _, ruta)) = cursantes.iter().find(|(cursante_id, _, _)| *cursante_id == id) {
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

/// Busca un cursante por nombre o ID (prefijo numérico).
fn buscar_cursante(ruta_cursantes: &Path, nombre: &str) -> Result<Option<PathBuf>, String> {
    let entradas = fs::read_dir(ruta_cursantes)
        .map_err(|e| format!("Error al leer directorio: {}", e))?;
    
    for entrada in entradas {
        let entrada = entrada.map_err(|e| format!("Error al leer entrada: {}", e))?;
        let ruta = entrada.path();
        
        if ruta.is_dir() {
            let nombre_dir = entrada.file_name();
            let nombre_str = nombre_dir.to_string_lossy();
            
            if nombre_str == nombre {
                return Ok(Some(ruta));
            }
            
            if let Some(id_str) = nombre_str.split('-').next() {
                if id_str == nombre {
                    return Ok(Some(ruta));
                }
            }
        }
    }
    
    Ok(None)
}