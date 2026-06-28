use std::fs;
use std::path::Path;
use serde_json::{json, Value};
use rustyline::DefaultEditor;
use crate::curso::preguntas::*;

/// Agrega cursante a un curso específico.
pub fn ejecutar(ruta_base: &Path, curso_arg: Option<&str>) -> Result<(), String> {
    let ruta_cursos = ruta_base.join("datos/cursos");
    
    if !ruta_cursos.exists() {
        return Err("No existe el directorio de cursos. Ejecute 'trazar inspector init' primero".to_string());
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
        println!("Use 'trazar curso nuevo' para crear un curso primero.");
        return Ok(());
    }
    
    cursos.sort_by_key(|(id, _)| *id);
    
    let mut rl = DefaultEditor::new()
        .map_err(|e| format!("Error al inicializar editor: {}", e))?;
    
    let (id_curso, nombre_curso) = if let Some(arg) = curso_arg {
        let nombre = encontrar_curso(&cursos, arg)?;
        let id = cursos.iter()
            .find(|(_, n)| n == &nombre)
            .map(|(id, _)| *id)
            .unwrap();
        (id, nombre)
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
        
        let nombre = encontrar_curso(&cursos, &id_seleccionado.to_string())?;
        (id_seleccionado, nombre)
    };
    
    let ruta_curso = ruta_cursos.join(&nombre_curso);
    let ruta_cursantes = ruta_curso.join("cursantes");
    
    if !ruta_cursantes.exists() {
        fs::create_dir_all(&ruta_cursantes)
            .map_err(|e| format!("Error al crear directorio de cursantes: {}", e))?;
    }
    
    let preguntas_json = include_str!("cursante-preguntas.json");
    let preguntas: Value = serde_json::from_str(preguntas_json)
        .map_err(|e| format!("Error al leer preguntas: {}", e))?;
    
    let campos = preguntas["campos"].as_array()
        .ok_or("Formato de preguntas inválido")?;
    
    println!("\nLas preguntas marcadas con * son obligatorias.\n");
    println!("Agregando cursante a: {}\n", nombre_curso);
    
    let mut respuestas = json!({});
    let mut nombre_cursante = String::new();
    
    for campo in campos {
        let campo_nombre = campo["campo"].as_str().unwrap();
        let mut mensaje = campo["mensaje"].as_str().unwrap().to_string();
        let tipo = campo["tipo"].as_str().unwrap();
        let obligatorio = campo["obligatorio"].as_bool().unwrap_or(false);
        
        if obligatorio {
            mensaje = format!("* {}", mensaje);
        }
        
        println!("{}", mensaje);
        
        let valor = match tipo {
            "str" => {
                if campo_nombre == "nombre" {
                    preguntar_nombre_cursante(&mut rl, obligatorio, &ruta_cursantes, None)?
                } else {
                    preguntar_texto(&mut rl, obligatorio)?
                }
            }
            "enum_int" => {
                let opciones = campo["opciones"].as_object()
                    .ok_or("Faltan opciones para enum")?;
                preguntar_enum(&mut rl, opciones, obligatorio)?
            }
            _ => return Err(format!("Tipo desconocido: {}", tipo)),
        };
        
        respuestas[campo_nombre] = valor;
        
        if campo_nombre == "nombre" {
            nombre_cursante = respuestas[campo_nombre].as_str().unwrap_or("").to_string();
        }
    }
    
    let id = generar_id_siguiente(&ruta_cursantes)?;
    let nombre_kebab = a_kebab_case(&nombre_cursante);
    let nombre_carpeta = format!("{:03}-{}", id, nombre_kebab);
    
    let ruta_nueva = ruta_cursantes.join(&nombre_carpeta);
    if ruta_nueva.exists() {
        return Err(format!("Ya existe cursante con nombre similar: {}", nombre_carpeta));
    }
    
    fs::create_dir_all(&ruta_nueva)
        .map_err(|e| format!("Error al crear directorio: {}", e))?;
    
    let mut json_final = json!({
        "modulo": "cursante",
        "version": 0,
        "id": id,
        "curso_id": id_curso,
        "curso_nombre": nombre_curso,
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
        }
        
        metadatos[campo_nombre] = meta;
    }
    
    json_final["metadatos_campos"] = metadatos;
    
    let ruta_info = ruta_nueva.join("cursante-info0.json");
    let contenido = serde_json::to_string_pretty(&json_final)
        .map_err(|e| format!("Error al serializar JSON: {}", e))?;
    
    fs::write(&ruta_info, contenido)
        .map_err(|e| format!("Error al escribir archivo: {}", e))?;
    
    println!("\n✓ Persona cursante creada: {}", nombre_carpeta);
    
    Ok(())
}

fn preguntar_nombre_cursante(
    rl: &mut DefaultEditor,
    obligatorio: bool,
    ruta_cursantes: &Path,
    excluir_id: Option<u32>,
) -> Result<Value, String> {
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
        
        let nombre_kebab = a_kebab_case(valor);
        
        if nombre_cursante_existe(ruta_cursantes, &nombre_kebab, excluir_id)? {
            println!("⚠ Ya existe cursante con el nombre '{}'. Ingrese otro nombre.", valor);
            continue;
        }
        
        let _ = rl.add_history_entry(&input);
        return Ok(json!(valor));
    }
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