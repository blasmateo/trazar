use std::fs;
use std::path::Path;
use serde_json::Value;
use rustyline::DefaultEditor;
use crate::curso::preguntas::*;

/// Muestra lista de cursantes de un curso o una persona cursante específica.
pub fn ejecutar(ruta_base: &Path, curso_arg: Option<&str>, cursante_arg: Option<&str>) -> Result<(), String> {
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
        println!("ℹ No hay cursos registrados.");
        return Ok(());
    }
    
    cursos.sort_by_key(|(id, _)| *id);
    
    let mut rl = DefaultEditor::new()
        .map_err(|e| format!("Error al inicializar editor: {}", e))?;
    
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
        println!("ℹ No hay cursantes con registro en '{}'.", nombre_curso);
        println!("Use 'trazar cursante -c {} nuevo' para agregar cursante.", nombre_curso.split('-').next().unwrap_or(""));
        return Ok(());
    }
    
    let mut cursantes: Vec<(u32, String)> = Vec::new();
    let entradas = fs::read_dir(&ruta_cursantes)
        .map_err(|e| format!("Error al leer directorio de cursantes: {}", e))?;
    
    for entrada in entradas {
        let entrada = entrada.map_err(|e| format!("Error al leer entrada: {}", e))?;
        let ruta = entrada.path();
        
        if ruta.is_dir() {
            if let Some(nombre) = ruta.file_name() {
                let nombre_str = nombre.to_string_lossy().to_string();
                if let Some(id_str) = nombre_str.split('-').next() {
                    if let Ok(id) = id_str.parse::<u32>() {
                        cursantes.push((id, nombre_str));
                    }
                }
            }
        }
    }
    
    if cursantes.is_empty() {
        println!("ℹ No hay cursantes con registro en '{}'.", nombre_curso);
        return Ok(());
    }
    
    cursantes.sort_by_key(|(id, _)| *id);
    
    let nombre_cursante = if let Some(arg) = cursante_arg {
        encontrar_cursante(&cursantes, arg)?
    } else {
        println!("\nCursantes con registro en '{}' ({}):", nombre_curso, cursantes.len());
        for (id, nombre) in &cursantes {
            println!("  [{}] {}", id, nombre);
        }
        
        let input = match rl.readline("\nSeleccione cursante por ID (o Enter para salir): ") {
            Ok(line) => line,
            Err(_) => return Ok(()),
        };
        
        let input = input.trim();
        if input.is_empty() {
            return Ok(());
        }
        
        let id_seleccionado: u32 = input.parse()
            .map_err(|_| "Se requiere ingresar un número válido".to_string())?;
        
        encontrar_cursante(&cursantes, &id_seleccionado.to_string())?
    };
    
    let ruta_cursante = ruta_cursantes.join(&nombre_cursante);
    mostrar_ficha_completa(&ruta_cursante, &nombre_curso)?;
    
    Ok(())
}

fn encontrar_cursante(cursantes: &[(u32, String)], argumento: &str) -> Result<String, String> {
    if let Ok(id) = argumento.parse::<u32>() {
        if let Some((_, nombre)) = cursantes.iter().find(|(cursante_id, _)| *cursante_id == id) {
            return Ok(nombre.clone());
        }
    }
    
    if let Some((_, nombre)) = cursantes.iter().find(|(_, n)| n == argumento) {
        return Ok(nombre.clone());
    }
    
    Err(format!("No se encontró cursante con el identificador '{}'", argumento))
}

fn mostrar_ficha_completa(ruta_cursante: &Path, nombre_curso: &str) -> Result<(), String> {
    let mut archivos_info: Vec<_> = fs::read_dir(ruta_cursante)
        .map_err(|e| format!("Error al leer directorio cursante: {}", e))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let nombre = e.file_name();
            let nombre_str = nombre.to_string_lossy();
            nombre_str.starts_with("cursante-info") && nombre_str.ends_with(".json")
        })
        .collect();
    
    archivos_info.sort_by(|a, b| {
        let num_a = extraer_numero(&a.file_name().to_string_lossy());
        let num_b = extraer_numero(&b.file_name().to_string_lossy());
        num_a.cmp(&num_b)
    });
    
    let nombre_cursante = ruta_cursante.file_name()
        .unwrap()
        .to_string_lossy();
    
    println!();
    println!("════════════════════════════════════════════════════════════");
    println!("  Cursante: {}", nombre_cursante);
    println!("  Curso: {}", nombre_curso);
    
    for archivo in archivos_info {
        let contenido = fs::read_to_string(archivo.path())
            .map_err(|e| format!("Error al leer {}: {}", archivo.path().display(), e))?;
        
        let json: Value = serde_json::from_str(&contenido)
            .map_err(|e| format!("Error al parsear JSON: {}", e))?;
        
        mostrar_ficha_ordenada(&json)?;
    }
    
    println!("────────────────────────────────────────────────────────────");
    println!("  Estructura:");
    mostrar_estructura(ruta_cursante, "  ")?;
    println!("════════════════════════════════════════════════════════════");
    
    Ok(())
}

fn extraer_numero(nombre: &str) -> u32 {
    nombre
        .trim_start_matches("cursante-info")
        .trim_end_matches(".json")
        .parse()
        .unwrap_or(0)
}

fn mostrar_ficha_ordenada(json: &Value) -> Result<(), String> {
    let orden_campos = vec![
        "id",
        "nombre",
        "email",
        "movil",
        "estado",
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

fn interpretar_valor(valor: &Value, metadato: Option<&Value>) -> String {
    if let Some(meta) = metadato {
        if let Some(tipo) = meta.get("tipo").and_then(|t| t.as_str()) {
            match tipo {
                "enum" => {
                    if let Some(val_str) = valor.as_u64().map(|v| v.to_string()) {
                        if let Some(opciones) = meta.get("valores") {
                            if let Some(texto) = opciones.get(&val_str).and_then(|v| v.as_str()) {
                                return texto.to_string();
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    
    formatear_valor(valor)
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