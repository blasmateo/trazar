use std::fs;
use std::path::Path;
use serde_json::{json, Value};
use rustyline::DefaultEditor;

use super::preguntas::*;

/// Agrega un nuevo curso de forma interactiva.
pub fn ejecutar(ruta_base: &Path) -> Result<(), String> {
    let ruta_cursos = ruta_base.join("datos/cursos");
    
    if !ruta_cursos.exists() {
        return Err("No existe el directorio de cursos. Ejecute 'trazar inspector init' primero".to_string());
    }
    
    let mut rl = DefaultEditor::new()
        .map_err(|e| format!("Error al inicializar editor: {}", e))?;
    
    let preguntas_json = include_str!("curso-preguntas.json");
    let preguntas: Value = serde_json::from_str(preguntas_json)
        .map_err(|e| format!("Error al leer preguntas: {}", e))?;
    
    let campos = preguntas["campos"].as_array()
        .ok_or("Formato de preguntas inválido")?;
    
    println!("\nLas preguntas marcadas con * son obligatorias.\n");
    
    let mut respuestas = json!({});
    let mut nombre_curso = String::new();
    
    for campo in campos {
        let campo_nombre = campo["campo"].as_str().unwrap();
        let mut mensaje = campo["mensaje"].as_str().unwrap().to_string();
        let tipo = campo["tipo"].as_str().unwrap();
        let obligatorio = campo["obligatorio"].as_bool().unwrap_or(false);
        
        if let Some(condicion) = campo.get("condicion") {
            if !evaluar_condicion(condicion, &respuestas) {
                continue;
            }
        }
        
        if obligatorio {
            mensaje = format!("* {}", mensaje);
        }
        
        println!("{}", mensaje);
        
        let valor = match tipo {
            "str" => {
				if campo_nombre == "nombre" {
					preguntar_nombre_curso(&mut rl, obligatorio, &ruta_cursos, None, "> ")?
				} else {
					preguntar_texto(&mut rl, obligatorio)?
				}
            }
            "int" => preguntar_numero(&mut rl, obligatorio)?,
            "float" => preguntar_float(&mut rl, obligatorio)?,
            "costo" => preguntar_costo(&mut rl, obligatorio)?,
            "fecha" => preguntar_fecha(&mut rl, obligatorio)?,
            "enum_int" => {
                let opciones = campo["opciones"].as_object()
                    .ok_or("Faltan opciones para enum")?;
                preguntar_enum(&mut rl, opciones, obligatorio)?
            }
            "array_str" => {
                let min_items = campo["min_items"].as_u64().unwrap_or(0) as usize;
                preguntar_array(&mut rl, obligatorio, min_items)?
            }
            _ => return Err(format!("Tipo desconocido: {}", tipo)),
        };
        
        respuestas[campo_nombre] = valor;
        
        if campo_nombre == "nombre" {
            nombre_curso = respuestas[campo_nombre].as_str().unwrap_or("").to_string();
        }
    }
    
    let id = generar_id_siguiente(&ruta_cursos)?;
    let nombre_kebab = a_kebab_case(&nombre_curso);
    let nombre_carpeta = format!("{:03}-{}", id, nombre_kebab);
    
    let ruta_nueva = ruta_cursos.join(&nombre_carpeta);
    if ruta_nueva.exists() {
        return Err(format!("Ya existe un curso con nombre similar: {}", nombre_carpeta));
    }
    
    fs::create_dir_all(&ruta_nueva)
        .map_err(|e| format!("Error al crear directorio: {}", e))?;
    
    let mut json_final = json!({
        "modulo": "curso",
        "version": 0,
        "id": id,
        "datos": respuestas.clone()
    });
    
    let mut metadatos = json!({});
    for campo in campos {
        let campo_nombre = campo["campo"].as_str().unwrap();
        let mut meta = json!({
            "etiqueta": campo["mensaje"]
        });
        
        if campo["tipo"] == "enum_int" {
            meta["tipo"] = json!("enum");
            meta["valores"] = campo["opciones"].clone();
        } else if campo_nombre == "costo" {
            let valor_costo = respuestas.get("costo");
            if let Some(v) = valor_costo {
                if v.is_number() {
                    meta["tipo"] = json!("moneda");
                    meta["simbolo"] = json!("$");
                } else if let Some(s) = v.as_str() {
                    meta["tipo"] = json!("texto_especial");
                    meta["valor_especial"] = json!(s);
                }
            }
        }
        
        metadatos[campo_nombre] = meta;
    }
    
    json_final["metadatos_campos"] = metadatos;
    
    let ruta_info = ruta_nueva.join("curso-info0.json");
    let contenido = serde_json::to_string_pretty(&json_final)
        .map_err(|e| format!("Error al serializar JSON: {}", e))?;
    
    fs::write(&ruta_info, contenido)
        .map_err(|e| format!("Error al escribir archivo: {}", e))?;
    
    println!("\n✓ Curso creado: {}", nombre_carpeta);
    
    Ok(())
}