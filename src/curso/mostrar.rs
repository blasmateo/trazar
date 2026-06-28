use std::fs;
use std::path::Path;
use serde_json::Value;
use rustyline::DefaultEditor;

use super::preguntas::*;

/// Muestra lista de cursos y permite seleccionar uno, o muestra directamente uno específico.
pub fn ejecutar(ruta_base: &Path, argumento: Option<&str>) -> Result<(), String> {
    let ruta_cursos = ruta_base.join("datos/cursos");
    
    if !ruta_cursos.exists() {
        println!("ℹ No hay cursos registrados.");
        println!("Use 'trazar curso nuevo' para agregar un curso.");
        return Ok(());
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
        println!("ℹ No hay cursos registrados.");
        println!("Use 'trazar curso nuevo' para agregar un curso.");
        return Ok(());
    }
    
    cursos.sort_by_key(|(id, _)| *id);
    
    let nombre_curso = if let Some(arg) = argumento {
        encontrar_curso(&cursos, arg)?
    } else {
        println!("Cursos registrados ({}):", cursos.len());
        for (id, nombre) in &cursos {
            println!("  [{}] {}", id, nombre);
        }
        
        let mut rl = match DefaultEditor::new() {
            Ok(editor) => editor,
            Err(e) => return Err(format!("Error al inicializar editor: {}", e)),
        };
        
        let input = match rl.readline("\nSeleccione un curso por ID (o Enter para salir): ") {
            Ok(line) => line,
            Err(_) => return Ok(()),
        };
        
        let input = input.trim();
        
        if input.is_empty() {
            return Ok(());
        }
        
        let id_seleccionado: u32 = input.parse()
            .map_err(|_| "Debe ingresar un número válido".to_string())?;
        
        encontrar_curso(&cursos, &id_seleccionado.to_string())?
    };
    
    let ruta_curso = ruta_cursos.join(&nombre_curso);
    mostrar_ficha_completa(&ruta_curso)?;
    
    Ok(())
}

fn mostrar_ficha_completa(ruta_curso: &Path) -> Result<(), String> {
    let mut archivos_info: Vec<_> = fs::read_dir(ruta_curso)
        .map_err(|e| format!("Error al leer directorio del curso: {}", e))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let nombre = e.file_name();
            let nombre_str = nombre.to_string_lossy();
            nombre_str.starts_with("curso-info") && nombre_str.ends_with(".json")
        })
        .collect();
    
    archivos_info.sort_by(|a, b| {
        let num_a = extraer_numero(&a.file_name().to_string_lossy());
        let num_b = extraer_numero(&b.file_name().to_string_lossy());
        num_a.cmp(&num_b)
    });
    
    let nombre_curso = ruta_curso.file_name()
        .unwrap()
        .to_string_lossy();
    
    println!();
    println!("════════════════════════════════════════════════════════════");
    println!("  Curso: {}", nombre_curso);
    
    for archivo in archivos_info {
        let contenido = fs::read_to_string(archivo.path())
            .map_err(|e| format!("Error al leer {}: {}", archivo.path().display(), e))?;
        
        let json: Value = serde_json::from_str(&contenido)
            .map_err(|e| format!("Error al parsear JSON: {}", e))?;
        
        mostrar_ficha_ordenada(&json)?;
    }
    
    println!("────────────────────────────────────────────────────────────");
    println!("  Estructura en datos/cursos:");
    mostrar_estructura(ruta_curso, "  ")?;
    println!("════════════════════════════════════════════════════════════");
    
    Ok(())
}

fn extraer_numero(nombre: &str) -> u32 {
    nombre
        .trim_start_matches("curso-info")
        .trim_end_matches(".json")
        .parse()
        .unwrap_or(0)
}

fn mostrar_ficha_ordenada(json: &Value) -> Result<(), String> {
    let orden_campos = vec![
        "id",
        "nombre",
        "estado",
        "costo",
        "cobranza",
        "docente",
        "soporte_tecnico",
        "descripcion",
        "fecha_inicio",
        "fecha_fin",
    ];
    
    println!("────────────────────────────────────────────────────────────");
    
    if let Some(metadatos) = json.get("metadatos_campos") {
        if let Some(datos) = json.get("datos") {
            if let Some(obj) = datos.as_object() {
                for campo in &orden_campos {
                    if let Some(valor) = obj.get(*campo) {
                        let etiqueta = metadatos.get(*campo)
                            .and_then(|m| m.get("etiqueta"))
                            .and_then(|e| e.as_str())
                            .unwrap_or(campo);
                        
                        let valor_str = interpretar_valor(valor, metadatos.get(*campo));
                        println!("  ▸ {}: {}", etiqueta, valor_str);
                    }
                }
                
                for (clave, valor) in obj {
                    if !orden_campos.contains(&clave.as_str()) {
                        let etiqueta = metadatos.get(clave)
                            .and_then(|m| m.get("etiqueta"))
                            .and_then(|e| e.as_str())
                            .unwrap_or(clave);
                        
                        let valor_str = interpretar_valor(valor, metadatos.get(clave));
                        println!("  ▸ {}: {}", etiqueta, valor_str);
                    }
                }
            }
        }
    } else {
        if let Some(obj) = json.as_object() {
            for campo in &orden_campos {
                if let Some(valor) = obj.get(*campo) {
                    if *campo != "modulo" && *campo != "version" {
                        println!("  ▸ {}: {}", campo, formatear_valor(valor));
                    }
                }
            }
            
            for (clave, valor) in obj {
                if !orden_campos.contains(&clave.as_str()) && clave != "modulo" && clave != "version" {
                    println!("  ▸ {}: {}", clave, formatear_valor(valor));
                }
            }
        }
    }
    
    Ok(())
}

fn mostrar_estructura(ruta: &Path, prefijo: &str) -> Result<(), String> {
    let mut entradas: Vec<_> = fs::read_dir(ruta)
        .map_err(|e| format!("Error al leer directorio: {}", e))?
        .filter_map(|e| e.ok())
        .collect();
    
    entradas.sort_by(|a, b| {
        let a_es_dir = a.path().is_dir();
        let b_es_dir = b.path().is_dir();
        b_es_dir.cmp(&a_es_dir).then_with(|| {
            a.file_name().cmp(&b.file_name())
        })
    });
    
    for (i, entrada) in entradas.iter().enumerate() {
        let es_ultimo = i == entradas.len() - 1;
        let conector = if es_ultimo { "└── " } else { "├── " };
        let nombre = entrada.file_name();
        
        println!("  {}{}{}", prefijo, conector, nombre.to_string_lossy());
        
        if entrada.path().is_dir() {
            let nuevo_prefijo = if es_ultimo {
                format!("{}    ", prefijo)
            } else {
                format!("{}│   ", prefijo)
            };
            mostrar_estructura(&entrada.path(), &nuevo_prefijo)?;
        }
    }
    
    Ok(())
}