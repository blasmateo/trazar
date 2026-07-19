use std::fs;
use std::path::{Path, PathBuf};
use rustyline::DefaultEditor;
use regex::Regex;

/// Importa uno o más archivos de asistencias a datos/cursos/<id-curso>/archivo/
/// 
/// - `archivos`: puede contener rutas de archivos individuales, globs (ej: *.txt), o directorios
/// - `bloque`: si es true, modo silencioso (falla si falta metadata). Si false, pregunta interactivamente
pub fn ejecutar(ruta_base: &Path, tipo_str: &str, archivos: &[String], bloque: bool) -> Result<(), String> {
    let tipo = super::TipoDataset::from_str(tipo_str)?;
    
    if !matches!(tipo, super::TipoDataset::Asistencias) {
        return Err(format!("Importación de '{}' aún no implementada. Solo 'asistencias' está disponible.", tipo_str));
    }
    
    let ruta_cursos = ruta_base.join("datos/cursos");
    if !ruta_cursos.exists() {
        return Err(format!("No existe el directorio {}. Ejecute 'trazar inspector init' primero.", ruta_cursos.display()));
    }
    
    let archivos_expandidos = expandir_argumentos(archivos)?;
    
    if archivos_expandidos.is_empty() {
        return Err("No se encontraron archivos para importar.".to_string());
    }
    
    // Primero, validar todos los archivos para clasificar errores de formato vs errores de metadata
    let mut archivos_validos: Vec<(String, MetadataAsistencias)> = Vec::new();
    let mut errores_formato: Vec<String> = Vec::new();
    let mut errores_metadata: Vec<String> = Vec::new();
    
    for ruta_str in &archivos_expandidos {
        let archivo_fuente = Path::new(ruta_str);
        
        if !archivo_fuente.exists() {
            errores_metadata.push(format!("{}: Archivo no encontrado", ruta_str));
            continue;
        }
        
        match validar_formato_archivo(archivo_fuente) {
            Ok(metadata) => {
                // Archivo con formato válido, guardamos metadata para procesar después
                archivos_validos.push((ruta_str.clone(), metadata));
            }
            Err(e) => {
                errores_formato.push(format!("{}: {}", ruta_str, e));
            }
        }
    }
    
    // Mostrar resumen de validación
	println!("");
    println!("╔══════════════════════════════════════════════════╗");
    println!("║              VALIDACIÓN PREVIA                   ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!("Total archivos: {}", archivos_expandidos.len());
    println!("Formato válido: {}, Errores de formato: {}, Errores de metadata: {}", 
             archivos_validos.len(), errores_formato.len(), errores_metadata.len());
    
    // Si no hay archivos válidos, terminar con error
    if archivos_validos.is_empty() {
        println!("\n✗ No hay archivos con formato válido para importar.");
        if !errores_metadata.is_empty() {
            for err in &errores_metadata {
                eprintln!("{}", err);
            }
        }
        if !errores_formato.is_empty() {
            for err in &errores_formato {
                eprintln!("{}", err);
            }
        }
        return Err("No se puede importar: todos los archivos tienen errores.".to_string());
    }
    
    // Preguntar si importar solo los válidos (cuando hay errores)
    if !errores_formato.is_empty() {
        println!("\n⚠ Se encontraron archivos con errores de formato:");
        for err in &errores_formato {
            println!("  {}", err);
        }
    }
    
    if !errores_metadata.is_empty() {
        println!("\n⚠ Se encontraron errores de metadata:");
        for err in &errores_metadata {
            println!("  {}", err);
        }
    }
    
    // Decidir si continuar con importación interactiva o bloque
    if bloque {
        // En modo bloque: importar automáticamente los archivos válidos
        if !errores_formato.is_empty() || !errores_metadata.is_empty() {
            println!("\n[Modo bloque] Continuando con importación automática...");
        }
    } else {
        // Modo interactivo: preguntar si hay errores
        if !errores_formato.is_empty() || !errores_metadata.is_empty() {
            let mut rl = DefaultEditor::new()
                .map_err(|e| format!("Error al inicializar editor: {}", e))?;
            
            let input = rl.readline("\n¿Desea importar solo los archivos válidos? (S/n): ")
                .map_err(|e| format!("Error al leer entrada: {}", e))?;
            
            if input.trim().to_lowercase() == "n" {
                println!("Operación cancelada.");
                return Ok(());
            }
        }
    }
    
    // Procesar archivos válidos
    let mut rl = if !bloque {
        Some(DefaultEditor::new()
            .map_err(|e| format!("Error al inicializar editor: {}", e))?)
    } else {
        None
    };
    
    if archivos_validos.len() > 1 {
		println!("");
        println!("╔══════════════════════════════════════════════════╗");
        println!("║           IMPORTACIÓN POR BLOQUE                 ║");
        println!("╚══════════════════════════════════════════════════╝");
		println!("");
    }
    
    let mut exitosos = 0;
    let mut errores = Vec::new();
    let mut saltados = 0;
    
    for (ruta_str, metadata) in &archivos_validos {
        let archivo_fuente = Path::new(ruta_str);
        
        match procesar_archivo_validado(
            ruta_base,
            archivo_fuente,
            metadata,
            bloque,
            rl.as_mut().unwrap_or(&mut DefaultEditor::new().unwrap()),
        ) {
            Ok(ruta_destino) => {
                println!("✓ {} → {}", archivo_fuente.display(), ruta_destino.display());
                exitosos += 1;
            }
            Err(e) => {
                if e == "Saltado por el usuario" {
                    saltados += 1;
                } else {
                    errores.push(format!("{}: {}", ruta_str, e));
                }
            }
        }
    }
    
    println!();
    println!("╔══════════════════════════════════════════════════╗");
    println!("║                  RESULTADOS                      ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!("Procesados: {} | Exitosos: {} | Saltados: {} | Errores: {}", 
             archivos_validos.len(), exitosos, saltados, errores.len() + errores_formato.len() + errores_metadata.len());
    
    if !errores.is_empty() {
        for err in &errores {
            eprintln!("✗ {}", err);
        }
    }
    
    // Mostrar errores de formato como advertencias (no impiden la importación exitosa)
    if !errores_formato.is_empty() {
        println!("\nAdvertencias (archivos no importados por formato inválido):");
        for err in &errores_formato {
            eprintln!("! {}", err);
        }
    }
    
    Ok(())
}

/// Validación estricta del formato del archivo (sin depender de cursos existentes)
fn validar_formato_archivo(archivo_fuente: &Path) -> Result<MetadataAsistencias, String> {
    let contenido = fs::read_to_string(archivo_fuente)
        .map_err(|e| format!("Error al leer archivo: {}", e))?;
    
    parsear_archivo_asistencias(&contenido)
}

/// Procesa un archivo YA VALIDADO (metadata ya obtenida)
fn procesar_archivo_validado(
    ruta_base: &Path,
    archivo_fuente: &Path,
    metadata: &MetadataAsistencias,
    bloque: bool,
    rl: &mut DefaultEditor,
) -> Result<PathBuf, String> {
    let curso_id = determinar_curso(ruta_base, metadata, rl, bloque)?;
    let asignatura_info = determinar_asignatura(ruta_base, &curso_id, metadata, rl, bloque)?;
    let clase_num = determinar_clase(metadata, archivo_fuente, rl, bloque)?;
    
    // Construir ruta destino: datos/cursos/<id-curso>/archivo/<tipo>/...
    let ruta_cursos = ruta_base.join("datos/cursos");
    let nombre_curso_dir = obtener_nombre_directorio_curso(&ruta_cursos, curso_id)?;
    let ruta_tipo = ruta_cursos.join(&nombre_curso_dir).join("archivo/asistencias");
    
    // Crear directorio si no existe
    if !ruta_tipo.exists() {
        fs::create_dir_all(&ruta_tipo)
            .map_err(|e| format!("Error al crear directorio {}: {}", ruta_tipo.display(), e))?;
        if !bloque {
            println!("✓ Creado directorio: {}", ruta_tipo.display());
        }
    }
    
    let ruta_destino = if let Some((asig_id, asig_nombre_kebab)) = &asignatura_info {
        let nombre_carpeta = format!("{:03}-{}", asig_id, asig_nombre_kebab);
        let ruta_carpeta = ruta_tipo.join(&nombre_carpeta);
        
        if !ruta_carpeta.exists() {
            fs::create_dir_all(&ruta_carpeta)
                .map_err(|e| format!("Error al crear directorio {}: {}", ruta_carpeta.display(), e))?;
            if !bloque {
                println!("✓ Creado: {}", nombre_carpeta);
            }
        }
        
        ruta_carpeta.join(format!("clase-{:03}.txt", clase_num))
    } else {
        ruta_tipo.join(format!("clase-{:03}.txt", clase_num))
    };
    
    if ruta_destino.exists() {
        if bloque {
            // En modo bloque: sobrescribir automáticamente
        } else {
            println!("\n⚠ Ya existe el archivo: {}", ruta_destino.display());
            println!("¿Qué desea hacer?");
            println!("  [1] Reemplazar el archivo existente");
            println!("  [2] Añadir a otra clase (especificar nuevo número de clase)");
            println!("  [3] Saltar este archivo");
            
            let input = rl.readline("\nSeleccione una opción (1/2/3): ")
                .map_err(|e| format!("Error al leer entrada: {}", e))?;
            
            match input.trim() {
                "1" => {
                    println!("Reemplazando archivo...");
                }
                "2" => {
                    let nueva_clase = rl.readline("Nuevo número de clase: ")
                        .map_err(|e| format!("Error al leer entrada: {}", e))?;
                    let nueva_clase_num: u32 = nueva_clase.trim().parse()
                        .map_err(|_| "Número de clase inválido".to_string())?;
                    
                    let nueva_ruta = if let Some((asig_id, asig_nombre_kebab)) = &asignatura_info {
                        let nombre_carpeta = format!("{:03}-{}", asig_id, asig_nombre_kebab);
                        ruta_tipo.join(&nombre_carpeta).join(format!("clase-{:03}.txt", nueva_clase_num))
                    } else {
                        ruta_tipo.join(format!("clase-{:03}.txt", nueva_clase_num))
                    };
                    
                    if nueva_ruta.exists() {
                        return Err(format!("Ya existe el archivo {} para la clase {}", nueva_ruta.display(), nueva_clase_num));
                    }
                    
                    fs::copy(archivo_fuente, &nueva_ruta)
                        .map_err(|e| format!("Error al copiar archivo: {}", e))?;
                    
                    println!("✓ Archivo importado: {}", nueva_ruta.display());
                    return Ok(nueva_ruta);
                }
                "3" | _ => {
                    println!("Archivo saltado.");
                    return Err("Saltado por el usuario".to_string());
                }
            }
        }
    }
    
    fs::copy(archivo_fuente, &ruta_destino)
        .map_err(|e| format!("Error al copiar archivo: {}", e))?;
    
    Ok(ruta_destino)
}

fn expandir_argumentos(archivos: &[String]) -> Result<Vec<String>, String> {
    let mut resultado = Vec::new();
    
    for arg in archivos {
        let path = Path::new(arg);
        
        if path.is_dir() {
            let entradas = fs::read_dir(path)
                .map_err(|e| format!("Error al leer directorio '{}': {}", arg, e))?;
            
            let mut encontrados = false;
            for entrada in entradas {
                let entrada = entrada.map_err(|e| format!("Error al leer entrada: {}", e))?;
                let ruta = entrada.path();
                if ruta.is_file() {
                    if let Some(ext) = ruta.extension() {
                        if ext == "txt" {
                            resultado.push(ruta.to_string_lossy().to_string());
                            encontrados = true;
                        }
                    }
                }
            }
            
            if !encontrados {
                return Err(format!("No se encontraron archivos .txt en el directorio '{}'", arg));
            }
        } else if path.is_file() {
            resultado.push(arg.clone());
        } else if arg.contains('*') || arg.contains('?') {
            match expandir_glob(arg) {
                Ok(mut archivos_glob) => {
                    if archivos_glob.is_empty() {
                        return Err(format!("No se encontraron archivos que coincidan con '{}'", arg));
                    }
                    resultado.append(&mut archivos_glob);
                }
                Err(e) => return Err(format!("Error al expandir '{}': {}", arg, e)),
            }
        } else {
            return Err(format!("'{}' no existe, no es un directorio, ni un patrón válido", arg));
        }
    }
    
    resultado.sort();
    resultado.dedup();
    
    Ok(resultado)
}

fn expandir_glob(patron: &str) -> Result<Vec<String>, String> {
    let path = Path::new(patron);
    
    let (dir, pattern_file) = if let Some(parent) = path.parent() {
        if parent.to_string_lossy().is_empty() {
            (PathBuf::from("."), patron.to_string())
        } else {
            let file_name = path.file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| patron.to_string());
            (parent.to_path_buf(), file_name)
        }
    } else {
        (PathBuf::from("."), patron.to_string())
    };
    
    if !dir.exists() {
        return Err(format!("Directorio '{}' no encontrado", dir.display()));
    }
    
    let regex_str = pattern_to_regex(&pattern_file);
    let re = Regex::new(&regex_str)
        .map_err(|e| format!("Error en patrón glob: {}", e))?;
    
    let entradas = fs::read_dir(&dir)
        .map_err(|e| format!("Error al leer directorio {}: {}", dir.display(), e))?;
    
    let mut resultados = Vec::new();
    for entrada in entradas {
        let entrada = entrada.map_err(|e| format!("Error al leer entrada: {}", e))?;
        let ruta = entrada.path();
        if ruta.is_file() {
            if let Some(nombre) = ruta.file_name() {
                let nombre_str = nombre.to_string_lossy();
                if re.is_match(&nombre_str) {
                    resultados.push(ruta.to_string_lossy().to_string());
                }
            }
        }
    }
    
    Ok(resultados)
}

fn pattern_to_regex(pattern: &str) -> String {
    let mut regex_str = String::from("^");
    for c in pattern.chars() {
        match c {
            '*' => regex_str.push_str(".*"),
            '?' => regex_str.push_str("."),
            '.' => regex_str.push_str("\\\\."),
            '\\' => regex_str.push_str("\\\\\\\\"),
            '+' => regex_str.push_str("\\\\+"),
            '(' => regex_str.push_str("\\\\("),
            ')' => regex_str.push_str("\\\\)"),
            '[' => regex_str.push_str("\\\\["),
            ']' => regex_str.push_str("\\\\]"),
            '{' => regex_str.push_str("\\\\{"),
            '}' => regex_str.push_str("\\\\}"),
            '^' => regex_str.push_str("\\\\^"),
            '$' => regex_str.push_str("\\\\$"),
            '|' => regex_str.push_str("\\\\|"),
            _ => regex_str.push(c),
        }
    }
    regex_str.push('$');
    regex_str
}

/// Obtiene el nombre del directorio del curso (ID-nombre) dado el cursoId
fn obtener_nombre_directorio_curso(ruta_cursos: &Path, curso_id: u32) -> Result<String, String> {
    if !ruta_cursos.exists() {
        return Err("No existe el directorio de cursos".to_string());
    }
    
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
                        if id == curso_id {
                            return Ok(nombre_str);
                        }
                    }
                }
            }
        }
    }
    
    Err(format!("Curso con ID {} no encontrado", curso_id))
}

struct MetadataAsistencias {
    curso_nombre: Option<String>,
    asignatura_nombre: Option<String>,
    clase_numero: Option<u32>,
}

fn a_kebab_case_estricto(texto: &str) -> String {
    texto
        .to_lowercase()
        .chars()
        .map(|c| {
            match c {
                'á' | 'à' | 'â' | 'ä' | 'ã' => 'a',
                'é' | 'è' | 'ê' | 'ë' => 'e',
                'í' | 'ì' | 'î' | 'ï' => 'i',
                'ó' | 'ò' | 'ô' | 'ö' | 'õ' => 'o',
                'ú' | 'ù' | 'û' | 'ü' => 'u',
                'ñ' => 'n',
                'ç' => 'c',
                _ if c.is_alphanumeric() => c,
                _ => '-',
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-")
}

/// Calcula similitud entre dos strings usando distancia de Levenshtein simplificada
fn similitud_coincidencia(nombre_curso: &str, nombre_directorio: &str) -> f64 {
    // Si son iguales, similitud 1.0
    if nombre_curso == nombre_directorio {
        return 1.0;
    }
    
    // Si el directorio contiene el nombre del curso (o viceversa)
    if nombre_curso.contains(nombre_directorio) || nombre_directorio.contains(nombre_curso) {
        return 0.9;
    }
    
    // Calcular coincidencia parcial por palabras clave
    let palabras_curso: Vec<&str> = nombre_curso.split('-').collect();
    let palabras_directorio: Vec<&str> = nombre_directorio.split('-').collect();
    
    let mut coincidencias = 0;
    for palabra in &palabras_curso {
        if palabras_directorio.contains(palabra) {
            coincidencias += 1;
        }
    }
    
    if palabras_curso.is_empty() {
        return 0.0;
    }
    
    // Si más del 80% de palabras coinciden, usar ese curso
    let ratio = coincidencias as f64 / palabras_curso.len() as f64;
    if ratio >= 0.8 {
        return ratio;
    }
    
    0.0
}

fn parsear_archivo_asistencias(contenido: &str) -> Result<MetadataAsistencias, String> {
    let mut metadata = MetadataAsistencias {
        curso_nombre: None,
        asignatura_nombre: None,
        clase_numero: None,
    };
    
    let mut en_cabeceras = true;
    let mut linea_numero = 0;
    let mut log_encontrado = false;
    
    for linea in contenido.lines() {
        linea_numero += 1;
        // Remover BOM invisible y caracteres de formato extraños al inicio
        let mut linea_limpia = linea.trim();
        // Remover BOM UTF-8 (\ufeff) si está presente
        if linea_limpia.starts_with('\u{feff}') {
            linea_limpia = &linea_limpia[3..];
        }
        // Remover espacio BOM común en algunos archivos
        if linea_limpia.starts_with('\u{00a0}') {
            linea_limpia = &linea_limpia[1..];
        }
        
        if linea_limpia.is_empty() {
            continue;
        }
        
        if en_cabeceras && linea_limpia.starts_with('#') {
            if linea_limpia.starts_with("# ====") {
                en_cabeceras = false;
                if !log_encontrado {
                    return Err("Error: Cabecera '# log: asistencias' es obligatoria".to_string());
                }
                continue;
            }
            
            if let Some(valor) = linea_limpia.strip_prefix("# log:") {
                let valor_trim = valor.trim();
                if valor_trim != "asistencias" {
                    return Err(format!("Línea {}: Cabecera 'log' debe ser 'asistencias', encontrado '{}'", linea_numero, valor_trim));
                }
                log_encontrado = true;
            } else if let Some(valor) = linea_limpia.strip_prefix("# curso:") {
                let valor_trim = valor.trim();
                if !valor_trim.is_empty() && valor_trim != "null" {
                    metadata.curso_nombre = Some(valor_trim.to_string());
                }
            } else if let Some(valor) = linea_limpia.strip_prefix("# asignatura:") {
                let valor_trim = valor.trim();
                if !valor_trim.is_empty() && valor_trim != "null" {
                    metadata.asignatura_nombre = Some(valor_trim.to_string());
                }
            } else if let Some(valor) = linea_limpia.strip_prefix("# clase:") {
                let valor_trim = valor.trim();
                if !valor_trim.is_empty() && valor_trim != "null" {
                    metadata.clase_numero = Some(valor_trim.parse()
                        .map_err(|_| format!("Línea {}: Número de clase inválido: '{}'", linea_numero, valor_trim))?);
                }
            } else if linea_limpia.starts_with("# fecha_creacion:") {
                if let Some(valor) = linea_limpia.strip_prefix("# fecha_creacion:") {
                    let valor_trim = valor.trim();
                    if !valor_trim.is_empty() && valor_trim != "null" {
                        let regex = regex::Regex::new(r"^\d{8}T\d{6}[+-]\d{4}$").unwrap();
                        if !regex.is_match(valor_trim) {
                            return Err(format!("Línea {}: Formato de fecha_creacion inválido. Debe ser YYYYMMDDTHHMMSS±Z (ej: 20260702T193230-0500)", linea_numero));
                        }
                    }
                }
            }
            continue;
        }
        
        if en_cabeceras {
            return Err(format!("Línea {}: Se esperaba separador de cabeceras '# ====' antes de las líneas de asistencia", linea_numero));
        }
        
        // Regex estricto: solo x/s/X/S con espacio y guión
        let regex = regex::Regex::new(r"^[XxSs]\s+-\s+.+$").unwrap();
        if !regex.is_match(linea_limpia) {
            return Err(format!("Línea {}: Formato inválido. Debe ser '[X|S] - <nombre>'. Encontrado: '{}'", linea_numero, linea_limpia));
        }
    }
    
    Ok(metadata)
}

fn determinar_curso(ruta_base: &Path, metadata: &MetadataAsistencias, rl: &mut DefaultEditor, bloque: bool) -> Result<u32, String> {
    let ruta_cursos = ruta_base.join("datos/cursos");
    
    if !ruta_cursos.exists() {
        return Err("No existe el directorio de cursos".to_string());
    }
    
    let mut cursos: Vec<(u32, String, String)> = Vec::new();
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
                        let nombre_sin_id = nombre_str.split('-').skip(1).collect::<Vec<_>>().join("-");
                        cursos.push((id, nombre_str, nombre_sin_id));
                    }
                }
            }
        }
    }
    
    if cursos.is_empty() {
        return Err("No hay cursos registrados".to_string());
    }
    
    cursos.sort_by_key(|(id, _, _)| *id);
    
    if let Some(nombre_curso) = &metadata.curso_nombre {
        let nombre_curso_kebab = a_kebab_case_estricto(nombre_curso);
        
        // Coincidencia exacta (kebab-case)
        if let Some((id, _, _)) = cursos.iter().find(|(_, _, nombre)| {
            let nombre_kebab = a_kebab_case_estricto(nombre);
            nombre_kebab == nombre_curso_kebab
        }) {
            return Ok(*id);
        }
        
        // No hay coincidencia exacta
    }
    
    if bloque {
        return Err("Modo bloque: no se pudo determinar el curso automáticamente. "
                   .to_owned() + "Use la cabecera '# curso:' en los archivos con el nombre exacto del curso.");
    }
    
    println!("\nCursos disponibles:");
    for (id, nombre_completo, _) in &cursos {
        println!("  [{}] {}", id, nombre_completo);
    }
    
    let input = rl.readline("\nSeleccione el ID del curso: ")
        .map_err(|e| format!("Error al leer entrada: {}", e))?;
    
    let id: u32 = input.trim().parse()
        .map_err(|_| "ID de curso inválido".to_string())?;
    
    if !cursos.iter().any(|(curso_id, _, _)| *curso_id == id) {
        return Err(format!("Curso con ID {} no encontrado", id));
    }
    
    Ok(id)
}

fn determinar_asignatura(
    ruta_base: &Path,
    curso_id: &u32,
    metadata: &MetadataAsistencias,
    rl: &mut DefaultEditor,
    bloque: bool,
) -> Result<Option<(u32, String)>, String> {
    let ruta_cursos = ruta_base.join("datos/cursos");
    
    let mut nombre_curso = None;
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
                        if id == *curso_id {
                            nombre_curso = Some(nombre_str);
                            break;
                        }
                    }
                }
            }
        }
    }
    
    let nombre_curso = nombre_curso.ok_or("Curso no encontrado")?;
    let ruta_asignaturas = ruta_cursos.join(&nombre_curso).join("asignaturas");
    
    if let Some(nombre_asignatura) = &metadata.asignatura_nombre {
        let regex_id = Regex::new(r"^(\d+)\s+(.+)$").unwrap();
        
        let (id_extraido, nombre_limpio) = if let Some(captures) = regex_id.captures(nombre_asignatura) {
            let id_str = captures.get(1).unwrap().as_str();
            let nombre_resto = captures.get(2).unwrap().as_str();
            let id: u32 = id_str.parse()
                .map_err(|_| format!("ID de asignatura inválido en el nombre: '{}'", id_str))?;
            (Some(id), nombre_resto.to_string())
        } else {
            (None, nombre_asignatura.clone())
        };
        
        let nombre_kebab = a_kebab_case_estricto(&nombre_limpio);
        
        if ruta_asignaturas.exists() {
            let mut asignaturas: Vec<(u32, String, String)> = Vec::new();
            let entradas = fs::read_dir(&ruta_asignaturas)
                .map_err(|e| format!("Error al leer directorio de asignaturas: {}", e))?;
            
            for entrada in entradas {
                let entrada = entrada.map_err(|e| format!("Error al leer entrada: {}", e))?;
                let ruta = entrada.path();
                
                if ruta.is_dir() {
                    if let Some(nombre) = ruta.file_name() {
                        let nombre_str = nombre.to_string_lossy().to_string();
                        if let Some(id_str) = nombre_str.split('-').next() {
                            if let Ok(id) = id_str.parse::<u32>() {
                                let nombre_sin_id = nombre_str.split('-').skip(1).collect::<Vec<_>>().join("-");
                                asignaturas.push((id, nombre_str, nombre_sin_id));
                            }
                        }
                    }
                }
            }
            
            asignaturas.sort_by_key(|(id, _, _)| *id);
            
            if let Some(id) = id_extraido {
                // Asignar nueva ID si no existe
                let asig_id = if asignaturas.iter().any(|(a, _, _)| *a == id) {
                    id
                } else {
                    // Nueva asignatura
                    let nueva_id = asignaturas.last()
                        .map(|(a, _, _)| a + 1)
                        .unwrap_or(1);
                    if !bloque {
                        println!("\n✓ Nueva asignatura detectada: {} (asignada ID {})", nombre_asignatura, nueva_id);
                    }
                    nueva_id
                };
                return Ok(Some((asig_id, nombre_kebab)));
            }
            
            // Intentar coincidencia parcial con asignaturas existentes
            let mejor_coincidencia = asignaturas.iter()
                .filter_map(|(id, _, nombre)| {
                    let similitud = similitud_coincidencia(&nombre_kebab, nombre);
                    if similitud > 0.0 {
                        Some((*id, similitud))
                    } else {
                        None
                    }
                })
                .max_by(|a, b| {
                    a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
                });
            
            if let Some((id, _)) = mejor_coincidencia {
                return Ok(Some((id, nombre_kebab)));
            }
            
            // Asignatura nueva, asignar ID automática
            let nueva_id = asignaturas.last()
                .map(|(a, _, _)| a + 1)
                .unwrap_or(1);
            if !bloque {
                println!("\n✓ Nueva asignatura: {} (ID {})", nombre_asignatura, nueva_id);
            }
            return Ok(Some((nueva_id, nombre_kebab)));
        }
        
        // No existen asignaturas, asignar ID 1
        if let Some(id) = id_extraido {
            return Ok(Some((id, nombre_kebab)));
        }
        
        if bloque {
            return Err("Modo bloque: no se pudo determinar el ID de la asignatura automáticamente. "
                       .to_owned() + "Incluya el ID al inicio del nombre (ej: '# asignatura: 1 Matematicas')");
        }
        
        println!("\nAsignatura detectada: {}", nombre_asignatura);
        println!("Nombre normalizado: {}", nombre_kebab);
        
        let input = rl.readline("Número de asignatura (o Enter para omitir): ")
            .map_err(|e| format!("Error al leer entrada: {}", e))?;
        
        let input_trim = input.trim();
        if input_trim.is_empty() {
            return Ok(None);
        }
        
        let id: u32 = input_trim.parse()
            .map_err(|_| "Número de asignatura inválido".to_string())?;
        
        return Ok(Some((id, nombre_kebab)));
    }
    
    Ok(None)
}

fn determinar_clase(metadata: &MetadataAsistencias, archivo_fuente: &Path, rl: &mut DefaultEditor, bloque: bool) -> Result<u32, String> {
    if let Some(clase) = metadata.clase_numero {
        return Ok(clase);
    }
    
    if let Some(nombre) = archivo_fuente.file_stem() {
        let nombre_str = nombre.to_string_lossy();
        let regex = Regex::new(r"c(\d{3})").unwrap();
        
        if let Some(captures) = regex.captures(&nombre_str) {
            if let Some(num_str) = captures.get(1) {
                return Ok(num_str.as_str().parse().unwrap());
            }
        }
    }
    
    if bloque {
        return Err("Modo bloque: no se pudo determinar el número de clase. "
                   .to_owned() + "Use la cabecera '# clase:' o nombre de archivo con patrón cNNN (ej: asistencias-c036.txt)");
    }
    
    let input = rl.readline("Número de clase: ")
        .map_err(|e| format!("Error al leer entrada: {}", e))?;
    
    input.trim().parse()
        .map_err(|_| "Número de clase inválido".to_string())
}