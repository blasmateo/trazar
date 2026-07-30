use std::fs;
use std::path::Path;
use serde_json::{json, Value};
use rustyline::DefaultEditor;
use chrono::{Local, NaiveDate, Offset as ChronoOffset};
use super::preguntas::*;

/// Edita los campos de un curso existente.
pub fn ejecutar(ruta_base: &Path, argumento: Option<&str>) -> Result<(), String> {
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
    
    let (id_curso, nombre_curso) = if let Some(arg) = argumento {
        let nombre = encontrar_curso(&cursos, arg)?;
        let id = cursos.iter()
            .find(|(_, n)| n == &nombre)
            .map(|(id, _)| *id)
            .unwrap();
        (id, nombre)
    } else {
        println!("Cursos registrados ({}):", cursos.len());
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
        
        let nombre = encontrar_curso(&cursos, &id_seleccionado.to_string())?;
        (id_seleccionado, nombre)
    };
    
    let ruta_curso = ruta_cursos.join(&nombre_curso);
    let ruta_info = ruta_curso.join("curso-info0.json");
    
    let contenido = fs::read_to_string(&ruta_info)
        .map_err(|e| format!("Error al leer archivo: {}", e))?;
    
    let mut json_curso: Value = serde_json::from_str(&contenido)
        .map_err(|e| format!("Error al parsear JSON: {}", e))?;

	// Extraer metadatos para interpretar valores
	let metadatos = json_curso.get("metadatos_campos").cloned();
    
    let datos = json_curso.get_mut("datos")
        .ok_or("JSON inválido: falta 'datos'")?;
    
    let preguntas_json = include_str!("curso-preguntas.json");
    let preguntas: Value = serde_json::from_str(preguntas_json)
        .map_err(|e| format!("Error al leer preguntas: {}", e))?;
    
    let campos = preguntas["campos"].as_array()
        .ok_or("Formato de preguntas inválido")?;
    
    loop {
        println!("\n════════════════════════════════════════════════════════════");
        println!("  Editando: {}", nombre_curso);
        println!("────────────────────────────────────────────────────────────");
        
        let mut campos_editables: Vec<(usize, &str, &str, String)> = Vec::new();
        let mut contador = 1;
        
        for campo in campos {
            let campo_nombre = campo["campo"].as_str().unwrap();
            let etiqueta = campo["mensaje"].as_str().unwrap();
            
            if campo_nombre == "id" {
                continue;
            }
            
            if let Some(condicion) = campo.get("condicion") {
                if !evaluar_condicion(condicion, datos) {
                    continue;
                }
            }
            
			let valor_actual = datos.get(campo_nombre).unwrap_or(&Value::Null);
			let valor_str = interpretar_valor(valor_actual, metadatos.as_ref().and_then(|m| m.get(campo_nombre)));
            
            println!("  [{}] {}: {}", contador, etiqueta, valor_str);
            campos_editables.push((contador, campo_nombre, etiqueta, valor_str));
            contador += 1;
        }
        
        println!("────────────────────────────────────────────────────────────");
        
        let input = match rl.readline("Seleccione el número del campo a editar (o Enter para guardar y salir): ") {
            Ok(line) => line,
            Err(_) => {
                println!("\nOperación cancelada. No se guardaron cambios.");
                return Ok(());
            }
        };
        
        let input = input.trim();
        
        if input.is_empty() {
            println!("\nSe guardarán los cambios en {}.", nombre_curso);
            let confirmacion = match rl.readline("Para confirmar, ingrese 'Si': ") {
                Ok(line) => line,
                Err(_) => {
                    println!("\nOperación cancelada. No se guardaron cambios.");
                    return Ok(());
                }
            };
            
            let conf = confirmacion.trim();
            if conf == "Si" {
                // Extraer nuevo nombre ANTES de serializar
                let nuevo_nombre_opt = datos.get("nombre")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                
                // Serializar JSON
                let contenido = serde_json::to_string_pretty(&json_curso)
                    .map_err(|e| format!("Error al serializar JSON: {}", e))?;
                
                // Escribir archivo
                fs::write(&ruta_info, contenido)
                    .map_err(|e| format!("Error al escribir archivo: {}", e))?;
                
                // Renombrar carpeta si cambió el nombre
                if let Some(nuevo_nombre) = nuevo_nombre_opt {
                    let nuevo_nombre_kebab = a_kebab_case(&nuevo_nombre);
                    let nuevo_nombre_carpeta = format!("{:03}-{}", id_curso, nuevo_nombre_kebab);
                    
                    if nuevo_nombre_carpeta != nombre_curso {
                        let nueva_ruta = ruta_cursos.join(&nuevo_nombre_carpeta);
                        fs::rename(&ruta_curso, &nueva_ruta)
                            .map_err(|e| format!("Error al renombrar carpeta: {}", e))?;
                        println!("✓ Curso renombrado a: {}", nuevo_nombre_carpeta);
                    }
                }
                
                println!("✓ Curso actualizado correctamente.");
                return Ok(());
            } else {
                println!("Operación cancelada. No se guardaron cambios.");
                return Ok(());
            }
        }
        
        let seleccion: usize = input.parse()
            .map_err(|_| "Se requiere ingresar un número válido".to_string())?;
        
        let campo_seleccionado = campos_editables.iter()
            .find(|(num, _, _, _)| *num == seleccion);
        
        let (_, campo_nombre, etiqueta, _) = match campo_seleccionado {
            Some(campo) => campo,
            None => {
                println!("⚠ Selección no válida. Intente nuevamente.");
                continue;
            }
        };
        
        let campo_config = campos.iter()
            .find(|c| c["campo"].as_str() == Some(campo_nombre))
            .ok_or("Campo no encontrado en configuración")?;
        
        let tipo = campo_config["tipo"].as_str().unwrap();
        let obligatorio = campo_config["obligatorio"].as_bool().unwrap_or(false);
        
        println!("\n▸ {}", etiqueta);
        println!("Valor actual: {}", interpretar_valor(datos.get(*campo_nombre).unwrap_or(&Value::Null), metadatos.as_ref().and_then(|m| m.get(*campo_nombre))));
        
        // Prompt según si es obligatorio u opcional
        let prompt = if obligatorio {
            "Valor (Enter=mantener): ".to_string()
        } else {
            "Valor ('vaciar' para borrar): ".to_string()
        };
        
        let valor = match tipo {
            "str" => {
                if *campo_nombre == "nombre" {
                    preguntar_nombre_curso(&mut rl, false, &ruta_cursos, Some(id_curso), &prompt)?
                } else {
                    preguntar_texto_con_valor_actual(&mut rl, datos.get(*campo_nombre), obligatorio, &prompt)?
                }
            }
            "int" => preguntar_numero_con_valor_actual(&mut rl, datos.get(*campo_nombre), obligatorio, &prompt)?,
            "float" => preguntar_float_con_valor_actual(&mut rl, datos.get(*campo_nombre), obligatorio, &prompt)?,
            "fecha" => preguntar_fecha_con_valor_actual(&mut rl, datos.get(*campo_nombre), obligatorio, &prompt)?,
            "enum_int" => {
                let opciones = campo_config["opciones"].as_object()
                    .ok_or("Faltan opciones para enum")?;
                preguntar_enum_con_valor_actual(&mut rl, opciones, datos.get(*campo_nombre), obligatorio, &prompt)?
            }
            "array_str" => {
                let min_items = campo_config["min_items"].as_u64().unwrap_or(0) as usize;
                preguntar_array_con_valor_actual(&mut rl, datos.get(*campo_nombre), min_items, obligatorio, &prompt)?
            }
            _ => return Err(format!("Tipo desconocido: {}", tipo)),
        };
        
        // Actualizar valor
        if !valor.is_null() || obligatorio {
            datos[*campo_nombre] = valor;
            println!("✓ Campo actualizado.");
        } else {
            println!("⚠ Este campo es obligatorio. Se mantiene el valor actual.");
        }
    }
}

fn preguntar_texto_con_valor_actual(rl: &mut DefaultEditor, valor_actual: Option<&Value>, obligatorio: bool, prompt: &str) -> Result<Value, String> {
    let input = rl.readline(prompt)
        .map_err(|e| format!("Error al leer entrada: {}", e))?;
    
    let valor = input.trim();
    
    if !obligatorio && valor.to_lowercase() == "vaciar" {
        return Ok(Value::Null);
    }
    
    if valor.is_empty() {
        return Ok(valor_actual.cloned().unwrap_or(Value::Null));
    }
    
    let _ = rl.add_history_entry(&input);
    Ok(json!(valor))
}

fn preguntar_numero_con_valor_actual(rl: &mut DefaultEditor, valor_actual: Option<&Value>, obligatorio: bool, prompt: &str) -> Result<Value, String> {
    let input = rl.readline(prompt)
        .map_err(|e| format!("Error al leer entrada: {}", e))?;
    
    let valor = input.trim();
    
    if !obligatorio && valor.to_lowercase() == "vaciar" {
        return Ok(Value::Null);
    }
    
    if valor.is_empty() {
        return Ok(valor_actual.cloned().unwrap_or(Value::Null));
    }
    
    match valor.parse::<i64>() {
        Ok(num) => {
            let _ = rl.add_history_entry(&input);
            Ok(json!(num))
        }
        Err(_) => {
            println!("Se requiere un número válido. Se mantiene el valor actual.");
            Ok(valor_actual.cloned().unwrap_or(Value::Null))
        }
    }
}

fn preguntar_float_con_valor_actual(rl: &mut DefaultEditor, valor_actual: Option<&Value>, obligatorio: bool, prompt: &str) -> Result<Value, String> {
    let input = rl.readline(prompt)
        .map_err(|e| format!("Error al leer entrada: {}", e))?;
    
    let valor = input.trim();
    
    if !obligatorio && valor.to_lowercase() == "vaciar" {
        return Ok(Value::Null);
    }
    
    if valor.is_empty() {
        return Ok(valor_actual.cloned().unwrap_or(Value::Null));
    }
    
    match valor.parse::<f64>() {
        Ok(num) => {
            let _ = rl.add_history_entry(&input);
            Ok(json!(num))
        }
        Err(_) => {
            println!("Se requiere un número válido. Se mantiene el valor actual.");
            Ok(valor_actual.cloned().unwrap_or(Value::Null))
        }
    }
}

fn preguntar_fecha_con_valor_actual(rl: &mut DefaultEditor, valor_actual: Option<&Value>, obligatorio: bool, prompt: &str) -> Result<Value, String> {
    let input = rl.readline(prompt)
        .map_err(|e| format!("Error al leer entrada: {}", e))?;
    
    let valor = input.trim();
    
    if !obligatorio && valor.to_lowercase() == "vaciar" {
        return Ok(Value::Null);
    }
    
    if valor.is_empty() {
        return Ok(valor_actual.cloned().unwrap_or(Value::Null));
    }
    
    match NaiveDate::parse_from_str(valor, "%Y-%m-%d") {
        Ok(fecha) => {
            let offset = Local::now().offset().fix();
            let fecha_completa = fecha.and_hms_opt(0, 0, 0).unwrap();
            let fecha_con_offset = fecha_completa.and_local_timezone(offset).unwrap();
            let fecha_str = fecha_con_offset.format("%Y-%m-%dT%H:%M:%S%:z").to_string();
            let _ = rl.add_history_entry(&input);
            Ok(json!(fecha_str))
        }
        Err(_) => {
            println!("Formato inválido. Se mantiene el valor actual.");
            Ok(valor_actual.cloned().unwrap_or(Value::Null))
        }
    }
}

fn preguntar_enum_con_valor_actual(
    rl: &mut DefaultEditor,
    opciones: &serde_json::Map<String, Value>,
    valor_actual: Option<&Value>,
    obligatorio: bool,
    prompt: &str,
) -> Result<Value, String> {
    println!("Opciones disponibles:");
    for (clave, valor) in opciones {
        println!("  {} = {}", clave, valor.as_str().unwrap());
    }
    
    let input = rl.readline(prompt)
        .map_err(|e| format!("Error al leer entrada: {}", e))?;
    
    let valor = input.trim();
    
    if !obligatorio && valor.to_lowercase() == "vaciar" {
        return Ok(Value::Null);
    }
    
    if valor.is_empty() {
        return Ok(valor_actual.cloned().unwrap_or(Value::Null));
    }
    
    if opciones.contains_key(valor) {
        match valor.parse::<i64>() {
            Ok(num) => {
                let _ = rl.add_history_entry(&input);
                Ok(json!(num))
            }
            Err(_) => {
                println!("Se requiere un número válido. Se mantiene el valor actual.");
                Ok(valor_actual.cloned().unwrap_or(Value::Null))
            }
        }
    } else {
        println!("Opción no válida. Se mantiene el valor actual.");
        Ok(valor_actual.cloned().unwrap_or(Value::Null))
    }
}

fn preguntar_array_con_valor_actual(
    rl: &mut DefaultEditor,
    valor_actual: Option<&Value>,
    min_items: usize,
    obligatorio: bool,
    prompt: &str,
) -> Result<Value, String> {
    let mut items = Vec::new();
    
    if obligatorio {
        println!("Ingrese uno por línea (Enter para terminar):");
    } else {
        println!("Ingrese uno por línea ('vaciar' para borrar todo):");
    }
    
    loop {
        let input = rl.readline(prompt)
            .map_err(|e| format!("Error al leer entrada: {}", e))?;
        
        let valor = input.trim();
        
        if !obligatorio && valor.to_lowercase() == "vaciar" && items.is_empty() {
            return Ok(Value::Null);
        }
        
        if valor.is_empty() {
            if items.is_empty() {
                return Ok(valor_actual.cloned().unwrap_or(Value::Null));
            }
            if items.len() < min_items {
                println!("Se requiere ingresar al menos {} elemento(s).", min_items);
                continue;
            }
            break;
        }
        
        let _ = rl.add_history_entry(&input);
        items.push(json!(valor));
    }
    
    Ok(json!(items))
}