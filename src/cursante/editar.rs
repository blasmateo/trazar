use std::fs;
use std::path::Path;
use serde_json::{json, Value};
use rustyline::DefaultEditor;
use chrono::{Local, NaiveDate, Offset as ChronoOffset};
use crate::curso::preguntas::*;

/// Edita los campos de un cursante existente.
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
            .map_err(|_| "Debe ingresar un número válido".to_string())?;
        
        encontrar_curso(&cursos, &id_seleccionado.to_string())?
    };
    
    let ruta_curso = ruta_cursos.join(&nombre_curso);
    let ruta_cursantes = ruta_curso.join("cursantes");
    
    if !ruta_cursantes.exists() {
        return Err(format!("No hay cursantes con registro en '{}'", nombre_curso));
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
        return Err(format!("No hay cursantes con registro en '{}'", nombre_curso));
    }
    
    cursantes.sort_by_key(|(id, _)| *id);
    
    let (id_cursante, nombre_cursante) = if let Some(arg) = cursante_arg {
        let nombre = encontrar_cursante(&cursantes, arg)?;
        let id = cursantes.iter()
            .find(|(_, n)| n == &nombre)
            .map(|(id, _)| *id)
            .unwrap();
        (id, nombre)
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
        
        let nombre = encontrar_cursante(&cursantes, &id_seleccionado.to_string())?;
        (id_seleccionado, nombre)
    };
    
    let ruta_cursante = ruta_cursantes.join(&nombre_cursante);
    let ruta_info = ruta_cursante.join("cursante-info0.json");
    
    let contenido = fs::read_to_string(&ruta_info)
        .map_err(|e| format!("Error al leer archivo: {}", e))?;
    
    let mut json_cursante: Value = serde_json::from_str(&contenido)
        .map_err(|e| format!("Error al parsear JSON: {}", e))?;
	
	// Extraer metadatos para interpretar valores
	let metadatos = json_cursante.get("metadatos_campos").cloned();

    let datos = json_cursante.get_mut("datos")
        .ok_or("JSON inválido: falta 'datos'")?;
    
    let preguntas_json = include_str!("cursante-preguntas.json");
    let preguntas: Value = serde_json::from_str(preguntas_json)
        .map_err(|e| format!("Error al leer preguntas: {}", e))?;
    
    let campos = preguntas["campos"].as_array()
        .ok_or("Formato de preguntas inválido")?;
    
    loop {
        println!("\n════════════════════════════════════════════════════════════");
        println!("  Editando: {}", nombre_cursante);
        println!("  Curso: {}", nombre_curso);
        println!("────────────────────────────────────────────────────────────");
        
        let mut campos_editables: Vec<(usize, &str, &str, String)> = Vec::new();
        let mut contador = 1;
        
        for campo in campos {
            let campo_nombre = campo["campo"].as_str().unwrap();
            let etiqueta = campo["mensaje"].as_str().unwrap();
            
            if campo_nombre == "id" {
                continue;
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
            println!("\nSe guardarán los cambios en {}.", nombre_cursante);
            let confirmacion = match rl.readline("Para confirmar, ingrese 'Si': ") {
                Ok(line) => line,
                Err(_) => {
                    println!("\nOperación cancelada. No se guardaron cambios.");
                    return Ok(());
                }
            };
            
            let conf = confirmacion.trim();
            if conf == "Si" {
                let nuevo_nombre_opt = datos.get("nombre")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                
                let contenido = serde_json::to_string_pretty(&json_cursante)
                    .map_err(|e| format!("Error al serializar JSON: {}", e))?;
                
                fs::write(&ruta_info, contenido)
                    .map_err(|e| format!("Error al escribir archivo: {}", e))?;
                
                if let Some(nuevo_nombre) = nuevo_nombre_opt {
                    let nuevo_nombre_kebab = a_kebab_case(&nuevo_nombre);
                    let nuevo_nombre_carpeta = format!("{:03}-{}", id_cursante, nuevo_nombre_kebab);
                    
                    if nuevo_nombre_carpeta != nombre_cursante {
                        let nueva_ruta = ruta_cursantes.join(&nuevo_nombre_carpeta);
                        fs::rename(&ruta_cursante, &nueva_ruta)
                            .map_err(|e| format!("Error al renombrar carpeta: {}", e))?;
                        println!("✓ Cursante se renombró a: {}", nuevo_nombre_carpeta);
                    }
                }
                
                println!("✓ Cursante se actualizó correctamente.");
                return Ok(());
            } else {
                println!("Operación cancelada. No se guardaron cambios.");
                return Ok(());
            }
        }
        
        let seleccion: usize = input.parse()
            .map_err(|_| "Debe ingresar un número válido".to_string())?;
        
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

		// Prompt corto según si es obligatorio u opcional
		let prompt = if obligatorio {
			"Valor (Enter=mantener): ".to_string()
		} else {
			"Valor ('vaciar' para borrar): ".to_string()
		};

		let valor = match tipo {
			"str" => {
				if *campo_nombre == "nombre" {
					preguntar_nombre_cursante_con_valor_actual(&mut rl, &ruta_cursantes, Some(id_cursante), &prompt)?
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
		if valor.is_null() && obligatorio {
			println!("⚠ Este campo es obligatorio. Se mantiene el valor actual.");
		} else {
			datos[*campo_nombre] = valor;
			println!("✓ Campo actualizado.");
		}
    }
}

fn encontrar_cursante(cursantes: &[(u32, String)], argumento: &str) -> Result<String, String> {
    // Buscar por ID numérico
    if let Ok(id) = argumento.parse::<u32>() {
        if let Some((_, nombre)) = cursantes.iter().find(|(cursante_id, _)| *cursante_id == id) {
            return Ok(nombre.clone());
        }
    }
    
    // Buscar por nombre exacto de carpeta
    if let Some((_, nombre)) = cursantes.iter().find(|(_, n)| n == argumento) {
        return Ok(nombre.clone());
    }
    
    // Buscar por nombre simple (ej: "maria-lopez" → "002-maria-lopez")
    let argumento_lower = argumento.to_lowercase();
    if let Some((_, nombre)) = cursantes.iter().find(|(_, n)| {
        n.to_lowercase().contains(&argumento_lower)
    }) {
        return Ok(nombre.clone());
    }
    
    // No encontrado: listar sugerencias
    let sugerencias: Vec<&str> = cursantes.iter()
        .map(|(_, n)| {
            n.splitn(2, '-').nth(1).unwrap_or(n)
        })
        .collect();
    
    Err(format!(
        "No se encontró cursante con el identificador '{}'.\n  Cursantes disponibles: {}",
        argumento,
        sugerencias.join(", ")
    ))
}

fn preguntar_nombre_cursante_con_valor_actual(
    rl: &mut DefaultEditor,
    ruta_cursantes: &Path,
    excluir_id: Option<u32>,
    prompt: &str,
) -> Result<Value, String> {
    let input = rl.readline(prompt)
        .map_err(|e| format!("Error al leer entrada: {}", e))?;
    
    let valor = input.trim();
    
    if valor.is_empty() {
        return Ok(Value::Null);
    }
    
    let nombre_kebab = a_kebab_case(valor);
    
    if nombre_cursante_existe(ruta_cursantes, &nombre_kebab, excluir_id)? {
        println!("⚠ Ya existe registro de cursante con el nombre '{}'. Se mantiene el valor actual.", valor);
        return Ok(Value::Null);
    }
    
    let _ = rl.add_history_entry(&input);
    Ok(json!(valor))
}

fn nombre_cursante_existe(ruta_cursantes: &Path, nombre_kebab: &str, excluir_id: Option<u32>) -> Result<bool, String> {
    if !ruta_cursantes.exists() {
        return Ok(false);
    }
    
    let entradas = fs::read_dir(ruta_cursantes)
        .map_err(|e| format!("Error al leer directorio de cursantes: {}", e))?;
    
    for entrada in entradas {
        let entrada = entrada.map_err(|e| format!("Error al leer entrada: {}", e))?;
        
        if entrada.path().is_dir() {
            let nombre_completo = entrada.file_name();
            let nombre_str = nombre_completo.to_string_lossy();
            
            let mut partes = nombre_str.splitn(2, '-');
            let id_str = partes.next().unwrap_or("");
            let nombre_sin_indice = partes.next().unwrap_or("").to_string();
            
            if let Some(excluir) = excluir_id {
                if let Ok(id) = id_str.parse::<u32>() {
                    if id == excluir {
                        continue;
                    }
                }
            }
            
            if nombre_sin_indice == nombre_kebab {
                return Ok(true);
            }
        }
    }
    
    Ok(false)
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