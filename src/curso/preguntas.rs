use std::fs;
use std::path::Path;
use serde_json::{json, Value};
use unicode_normalization::UnicodeNormalization;
use chrono::{Local, NaiveDate, Offset as ChronoOffset};
use rustyline::DefaultEditor;

/// Pregunta un texto genérico.
pub fn preguntar_texto(rl: &mut DefaultEditor, obligatorio: bool) -> Result<Value, String> {
    loop {
        let input = rl.readline("> ")
            .map_err(|e| format!("Error al leer entrada: {}", e))?;
        
        let valor = input.trim();
        
        if valor.is_empty() {
            if obligatorio {
                println!("Este campo es obligatorio. Intente nuevamente.");
                continue;
            } else {
                return Ok(Value::Null);
            }
        }
        
        let _ = rl.add_history_entry(&input);
        return Ok(json!(valor));
    }
}

/// Pregunta el nombre del curso y valida que no exista.
pub fn preguntar_nombre_curso(
    rl: &mut DefaultEditor,
    obligatorio: bool,
    ruta_cursos: &Path,
    excluir_id: Option<u32>,
    prompt: &str,
) -> Result<Value, String> {
    loop {
        let input = rl.readline(prompt)
            .map_err(|e| format!("Error al leer entrada: {}", e))?;
        
        let valor = input.trim();
        
        if valor.is_empty() {
            if obligatorio {
                println!("Este campo es obligatorio. Intente nuevamente.");
                continue;
            } else {
                return Ok(Value::Null);
            }
        }
        
        let nombre_kebab = a_kebab_case(valor);
        
        if nombre_existe(ruta_cursos, &nombre_kebab, excluir_id)? {
            println!("⚠ Ya existe un curso con el nombre '{}'. Ingrese otro nombre.", valor);
            continue;
        }
        
        let _ = rl.add_history_entry(&input);
        return Ok(json!(valor));
    }
}

/// Pregunta un número entero.
pub fn preguntar_numero(rl: &mut DefaultEditor, obligatorio: bool) -> Result<Value, String> {
    loop {
        let input = rl.readline("> ")
            .map_err(|e| format!("Error al leer entrada: {}", e))?;
        
        let valor = input.trim();
        
        if valor.is_empty() {
            if obligatorio {
                println!("Este campo es obligatorio. Intente nuevamente.");
                continue;
            } else {
                return Ok(Value::Null);
            }
        }
        
        match valor.parse::<i64>() {
            Ok(num) => {
                let _ = rl.add_history_entry(&input);
                return Ok(json!(num));
            }
            Err(_) => println!("Se requiere ingresar un número válido. Intente nuevamente."),
        }
    }
}

/// Pregunta un valor de costo: número, "variable", "cantidad voluntaria" o "no aplica".
pub fn preguntar_costo(rl: &mut DefaultEditor, obligatorio: bool) -> Result<Value, String> {
    loop {
        let input = rl.readline("> ")
            .map_err(|e| format!("Error al leer entrada: {}", e))?;
        
        let valor = input.trim().to_lowercase();
        
        if valor.is_empty() {
            if obligatorio {
                println!("Este campo es obligatorio. Intente nuevamente.");
                continue;
            } else {
                return Ok(Value::Null);
            }
        }
        
        // Valores especiales de texto
        if valor == "variable" || valor == "cantidad voluntaria" || valor == "no aplica" {
            let _ = rl.add_history_entry(&input);
            return Ok(json!(valor));
        }
        
        // Número (con o sin decimales)
        match valor.parse::<f64>() {
            Ok(num) => {
                if num >= 0.0 {
                    let _ = rl.add_history_entry(&input);
                    return Ok(json!(num));
                } else {
                    println!("El monto no puede ser negativo. Intente nuevamente.");
                }
            }
            Err(_) => {
                println!("Valor inválido. Use un número, 'variable', 'cantidad voluntaria' o 'no aplica'.");
            }
        }
    }
}

/// Pregunta un número con decimales.
pub fn preguntar_float(rl: &mut DefaultEditor, obligatorio: bool) -> Result<Value, String> {
    loop {
        let input = rl.readline("> ")
            .map_err(|e| format!("Error al leer entrada: {}", e))?;
        
        let valor = input.trim();
        
        if valor.is_empty() {
            if obligatorio {
                println!("Este campo es obligatorio. Intente nuevamente.");
                continue;
            } else {
                return Ok(Value::Null);
            }
        }
        
        match valor.parse::<f64>() {
            Ok(num) => {
                let _ = rl.add_history_entry(&input);
                return Ok(json!(num));
            }
            Err(_) => println!("Se requiere ingresar un número válido (puede incluir decimales). Intente nuevamente."),
        }
    }
}

/// Pregunta una fecha en formato YYYY-MM-DD.
pub fn preguntar_fecha(rl: &mut DefaultEditor, obligatorio: bool) -> Result<Value, String> {
    loop {
        let input = rl.readline("> ")
            .map_err(|e| format!("Error al leer entrada: {}", e))?;
        
        let valor = input.trim();
        
        if valor.is_empty() {
            if obligatorio {
                println!("Este campo es obligatorio. Intente nuevamente.");
                continue;
            } else {
                return Ok(Value::Null);
            }
        }
        
        match NaiveDate::parse_from_str(valor, "%Y-%m-%d") {
            Ok(fecha) => {
                let offset = Local::now().offset().fix();
                let fecha_completa = fecha.and_hms_opt(0, 0, 0).unwrap();
                let fecha_con_offset = fecha_completa.and_local_timezone(offset).unwrap();
                let fecha_str = fecha_con_offset.format("%Y-%m-%dT%H:%M:%S%:z").to_string();
                let _ = rl.add_history_entry(&input);
                return Ok(json!(fecha_str));
            }
            Err(_) => println!("Formato inválido. Use YYYY-MM-DD (ejemplo: 2026-07-01). Intente nuevamente."),
        }
    }
}

/// Pregunta un valor de enum.
pub fn preguntar_enum(rl: &mut DefaultEditor, opciones: &serde_json::Map<String, Value>, obligatorio: bool) -> Result<Value, String> {
    println!("Opciones disponibles:");
    for (clave, valor) in opciones {
        println!("  {} = {}", clave, valor.as_str().unwrap());
    }
    
    loop {
        let input = rl.readline("> ")
            .map_err(|e| format!("Error al leer entrada: {}", e))?;
        
        let valor = input.trim();
        
        if valor.is_empty() {
            if obligatorio {
                println!("Este campo es obligatorio. Intente nuevamente.");
                continue;
            } else {
                return Ok(Value::Null);
            }
        }
        
        if opciones.contains_key(valor) {
            match valor.parse::<i64>() {
                Ok(num) => {
                    let _ = rl.add_history_entry(&input);
                    return Ok(json!(num));
                }
                Err(_) => println!("Se requiere ingresar un número válido."),
            }
        } else {
            println!("Opción no válida. Intente nuevamente.");
        }
    }
}

/// Pregunta un array de strings.
pub fn preguntar_array(rl: &mut DefaultEditor, obligatorio: bool, min_items: usize) -> Result<Value, String> {
    let mut items = Vec::new();
    
    println!("Ingrese uno por línea (deje vacío para terminar):");
    
    loop {
        let input = rl.readline("> ")
            .map_err(|e| format!("Error al leer entrada: {}", e))?;
        
        let valor = input.trim();
        
        if valor.is_empty() {
            if items.len() < min_items {
                println!("Se requiere ingresar al menos {} elemento(s).", min_items);
                continue;
            }
            break;
        }
        
        let _ = rl.add_history_entry(&input);
        items.push(json!(valor));
    }
    
    if items.is_empty() && obligatorio {
        return Ok(Value::Null);
    }
    
    Ok(json!(items))
}

/// Evalúa una condición contra las respuestas actuales.
pub fn evaluar_condicion(condicion: &Value, respuestas: &Value) -> bool {
    let campo = match condicion.get("campo").and_then(|c| c.as_str()) {
        Some(c) => c,
        None => return true,
    };
    
    let operador = match condicion.get("operador").and_then(|o| o.as_str()) {
        Some(o) => o,
        None => return true,
    };
    
    let valor_esperado = match condicion.get("valor") {
        Some(v) => v,
        None => return true,
    };
    
    let valor_actual = match respuestas.get(campo) {
        Some(v) => v,
        None => return true,
    };
    
    match operador {
        ">" => {
            let actual = valor_actual.as_f64().unwrap_or(0.0);
            let esperado = valor_esperado.as_f64().unwrap_or(0.0);
            actual > esperado
        }
        "<" => {
            let actual = valor_actual.as_f64().unwrap_or(0.0);
            let esperado = valor_esperado.as_f64().unwrap_or(0.0);
            actual < esperado
        }
        "==" => valor_actual == valor_esperado,
        "!=" => valor_actual != valor_esperado,
        "no_es_no_aplica" => {
            if let Some(s) = valor_actual.as_str() {
                s != "no aplica"
            } else {
                // Si es número (costo numérico > 0), se pregunta periodicidad
                valor_actual.as_f64().unwrap_or(0.0) >= 0.0
            }
        }
        ">=" => {
            let actual = valor_actual.as_f64().unwrap_or(0.0);
            let esperado = valor_esperado.as_f64().unwrap_or(0.0);
            actual >= esperado
        }
        "<=" => {
            let actual = valor_actual.as_f64().unwrap_or(0.0);
            let esperado = valor_esperado.as_f64().unwrap_or(0.0);
            actual <= esperado
        }
        _ => true,
    }
}

/// Genera el siguiente ID secuencial.
pub fn generar_id_siguiente(ruta_cursos: &Path) -> Result<u32, String> {
    let mut max_id: u32 = 0;
    
    let entradas = fs::read_dir(ruta_cursos)
        .map_err(|e| format!("Error al leer directorio: {}", e))?;
    
    for entrada in entradas {
        let entrada = entrada.map_err(|e| format!("Error al leer entrada: {}", e))?;
        
        if entrada.path().is_dir() {
            let nombre = entrada.file_name();
            let nombre_str = nombre.to_string_lossy();
            if let Some(id_str) = nombre_str.split('-').next() {
                if let Ok(id) = id_str.parse::<u32>() {
                    if id > max_id {
                        max_id = id;
                    }
                }
            }
        }
    }
    
    Ok(max_id + 1)
}

/// Verifica si ya existe un curso con el nombre dado (sin importar el índice).
pub fn nombre_existe(ruta_cursos: &Path, nombre_kebab: &str, excluir_id: Option<u32>) -> Result<bool, String> {
    let entradas = fs::read_dir(ruta_cursos)
        .map_err(|e| format!("Error al leer directorio de cursos: {}", e))?;
    
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

/// Convierte texto a kebab-case (minúsculas, sin tildes, sin ñ).
pub fn a_kebab_case(texto: &str) -> String {
    let normalizado: String = texto.nfkd().filter(|c| c.is_ascii()).collect();
    
    normalizado
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-")
}

/// Busca un curso por ID o nombre completo en la lista.
pub fn encontrar_curso(cursos: &[(u32, String)], argumento: &str) -> Result<String, String> {
    if let Ok(id) = argumento.parse::<u32>() {
        if let Some((_, nombre)) = cursos.iter().find(|(curso_id, _)| *curso_id == id) {
            return Ok(nombre.clone());
        }
    }
    
    if let Some((_, nombre)) = cursos.iter().find(|(_, n)| n == argumento) {
        return Ok(nombre.clone());
    }
    
    Err(format!("No se encontró ningún curso con el identificador '{}'", argumento))
}

/// Interpreta un valor según sus metadatos (enum, moneda, etc).
pub fn interpretar_valor(valor: &Value, metadato: Option<&Value>) -> String {
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
                "moneda" => {
                    if let Some(num) = valor.as_f64() {
                        let simbolo = meta.get("simbolo")
                            .and_then(|s| s.as_str())
                            .unwrap_or("$");
                        return format!("{} {:.2}", simbolo, num);
                    }
                }
                _ => {}
            }
        }
    }
    
    formatear_valor(valor)
}

/// Formatea un valor para mostrarlo.
pub fn formatear_valor(valor: &Value) -> String {
    match valor {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => if *b { "sí" } else { "no" }.to_string(),
        Value::Array(arr) => {
            if arr.is_empty() {
                "(vacío)".to_string()
            } else {
                let items: Vec<String> = arr.iter()
                    .map(|v| formatear_valor(v))
                    .collect();
                items.join(", ")
            }
        }
        Value::Null => "(vacío)".to_string(),
        _ => valor.to_string(),
    }
}