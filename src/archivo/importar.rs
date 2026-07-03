use std::fs;
use std::path::{Path, PathBuf};
use rustyline::DefaultEditor;
use regex::Regex;

/// Importa uno o más archivos de asistencias a datos/archivo/asistencias/
/// 
/// - `archivos`: puede contener rutas de archivos individuales, globs (ej: *.txt), o directorios
/// - `bloque`: si es true, procesa sin preguntas interactivas (falla si falta metadata)
pub fn ejecutar(ruta_base: &Path, tipo_str: &str, archivos: &[String], bloque: bool) -> Result<(), String> {
    let tipo = super::TipoDataset::from_str(tipo_str)?;
    
    // Solo asistencias está implementado por ahora
    if !matches!(tipo, super::TipoDataset::Asistencias) {
        return Err(format!("Importación de '{}' aún no implementada. Solo 'asistencias' está disponible.", tipo_str));
    }
    
    // Verificar que existe el directorio destino
    let ruta_archivo = ruta_base.join("datos/archivo");
    let ruta_tipo = ruta_archivo.join(tipo.nombre_directorio());
    if !ruta_tipo.exists() {
        return Err(format!("No existe el directorio {}. Ejecute 'trazar inspector init' primero.", ruta_tipo.display()));
    }
    
    // Expandir argumentos: archivos, globs, directorios -> lista plana de archivos
    let archivos_expandidos = expandir_argumentos(archivos)?;
    
    if archivos_expandidos.is_empty() {
        return Err("No se encontraron archivos para importar.".to_string());
    }
    
    if archivos_expandidos.len() > 1 {
        println!("╔══════════════════════════════════════════════════╗");
        println!("║           IMPORTACIÓN POR BLOQUE                ║");
        println!("╚══════════════════════════════════════════════════╝");
        println!("Archivos a procesar: {}\n", archivos_expandidos.len());
    }
    
    // En modo bloque, si hay metadata completa, aplicar a todos
    let mut bloque_curso_id: Option<u32> = None;
    let mut bloque_asignatura: Option<(u32, String)> = None;
    
    // Solo crear editor interactivo si no estamos en modo bloque
    let mut rl_opt: Option<DefaultEditor> = if !bloque {
        Some(DefaultEditor::new()
            .map_err(|e| format!("Error al inicializar editor: {}", e))?)
    } else {
        None
    };
    
    // Contadores
    let mut exitosos = 0;
    let mut errores = Vec::new();
    
    for ruta_str in &archivos_expandidos {
        let archivo_fuente = Path::new(ruta_str);
        
        if !archivo_fuente.exists() {
            errores.push(format!("{}: Archivo no encontrado", ruta_str));
            continue;
        }
        
        match procesar_archivo(
            ruta_base,
            &ruta_tipo,
            archivo_fuente,
            bloque,
            &mut bloque_curso_id,
            &mut bloque_asignatura,
            &mut rl_opt,
        ) {
            Ok(ruta_destino) => {
                println!("✓ {} → {}", archivo_fuente.display(), ruta_destino.display());
                exitosos += 1;
            }
            Err(e) => {
                errores.push(format!("{}: {}", ruta_str, e));
            }
        }
    }
    
    // Reporte final
    println!();
    if archivos_expandidos.len() > 1 {
        println!("╔══════════════════════════════════════════════════╗");
        println!("║                  RESULTADOS                      ║");
        println!("╚══════════════════════════════════════════════════╝");
    }
    println!("Procesados: {} | Exitosos: {} | Errores: {}", 
             archivos_expandidos.len(), exitosos, errores.len());
    
    if !errores.is_empty() {
        for err in &errores {
            eprintln!("✗ {}", err);
        }
        return Err(format!("{} archivo(s) con errores", errores.len()));
    }
    
    Ok(())
}

/// Expande los argumentos de entrada: archivos, globs (*.txt), o directorios
fn expandir_argumentos(archivos: &[String]) -> Result<Vec<String>, String> {
    let mut resultado = Vec::new();
    
    for arg in archivos {
        let path = Path::new(arg);
        
        if path.is_dir() {
            // Es un directorio: listar archivos .txt dentro
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
            // Es un archivo directo
            resultado.push(arg.clone());
        } else if arg.contains('*') || arg.contains('?') {
            // Es un glob pattern: expandir con shell (delegamos a glob)
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
    
    // Ordenar y deduplicar
    resultado.sort();
    resultado.dedup();
    
    Ok(resultado)
}

/// Expande un glob pattern usando el crate glob o implementación manual
fn expandir_glob(patron: &str) -> Result<Vec<String>, String> {
    let path = Path::new(patron);
    
    // Obtener el directorio base del patrón
    let (dir, pattern_file) = if let Some(parent) = path.parent() {
        if parent.to_string_lossy().is_empty() {
            (PathBuf::from("."), patron.to_string())
        } else {
            // Extraer el nombre del archivo del patrón
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
    
    // Convertir patrón glob a regex
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

/// Convierte un patrón glob simple a regex
fn pattern_to_regex(pattern: &str) -> String {
    let mut regex_str = String::from("^");
    for c in pattern.chars() {
        match c {
            '*' => regex_str.push_str(".*"),
            '?' => regex_str.push_str("."),
            '.' => regex_str.push_str("\\."),
            '\\' => regex_str.push_str("\\\\"),
            '+' => regex_str.push_str("\\+"),
            '(' => regex_str.push_str("\\("),
            ')' => regex_str.push_str("\\)"),
            '[' => regex_str.push_str("\\["),
            ']' => regex_str.push_str("\\]"),
            '{' => regex_str.push_str("\\{"),
            '}' => regex_str.push_str("\\}"),
            '^' => regex_str.push_str("\\^"),
            '$' => regex_str.push_str("\\$"),
            '|' => regex_str.push_str("\\|"),
            _ => regex_str.push(c),
        }
    }
    regex_str.push('$');
    regex_str
}

/// Procesa un archivo individual de asistencias
fn procesar_archivo(
    ruta_base: &Path,
    ruta_tipo: &Path,
    archivo_fuente: &Path,
    bloque: bool,
    bloque_curso_id: &mut Option<u32>,
    bloque_asignatura: &mut Option<(u32, String)>,
    rl: &mut Option<DefaultEditor>,
) -> Result<PathBuf, String> {
    // Leer contenido del archivo
    let contenido = fs::read_to_string(archivo_fuente)
        .map_err(|e| format!("Error al leer archivo: {}", e))?;
    
    // Parsear cabeceras y validar formato
    let metadata = parsear_archivo_asistencias(&contenido)?;
    
    // Determinar curso (cada bloque termina el préstamo antes del siguiente)
    let curso_id = if let Some(id) = *bloque_curso_id {
        id
    } else {
        let id;
        // Bloque separado para que el préstamo de rl termine antes de la siguiente llamada
        {
            let mut rl_ref = rl.as_mut();
            id = determinar_curso(ruta_base, &metadata, &mut rl_ref, bloque)?;
        }
        if bloque {
            *bloque_curso_id = Some(id);
        }
        id
    };
    
    // Determinar asignatura
    let asignatura_info = if let Some(ref info) = *bloque_asignatura {
        Some(info.clone())
    } else {
        let info;
        {
            let mut rl_ref = rl.as_mut();
            info = determinar_asignatura(ruta_base, &curso_id, &metadata, &mut rl_ref, bloque)?;
        }
        if bloque && info.is_some() {
            *bloque_asignatura = info.clone();
        }
        info
    };
    
    // Determinar número de clase
    let clase_num;
    {
        let mut rl_ref = rl.as_mut();
        clase_num = determinar_clase(&metadata, archivo_fuente, &mut rl_ref, bloque)?;
    }
    
    // Generar ruta destino según si hay asignatura o no
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
        ruta_tipo.join(format!("curso-{:03}_clase-{:03}.txt", curso_id, clase_num))
    };
    
    // Verificar si el archivo destino ya existe
    if ruta_destino.exists() {
        if bloque {
            // En modo bloque: sobrescribir automáticamente
            // (no preguntar)
        } else {
            let rl = rl.as_mut().unwrap();
            
            println!("\n⚠ Ya existe el archivo: {}", ruta_destino.display());
            println!("¿Qué desea hacer?");
            println!("  [1] Reemplazar el archivo existente");
            println!("  [2] Añadir a otra clase (especificar nuevo número de clase)");
            println!("  [3] Saltar este archivo");
            
            let input = rl.readline("\nSeleccione una opción (1/2/3): ")
                .map_err(|e| format!("Error al leer entrada: {}", e))?;
            
            match input.trim() {
                "1" => {
                    // Reemplazar
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
                        ruta_tipo.join(format!("curso-{:03}_clase-{:03}.txt", curso_id, nueva_clase_num))
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
    
    // Copiar archivo al destino
    fs::copy(archivo_fuente, &ruta_destino)
        .map_err(|e| format!("Error al copiar archivo: {}", e))?;
    
    Ok(ruta_destino)
}

/// Metadata extraída del archivo de asistencias
struct MetadataAsistencias {
    curso_nombre: Option<String>,
    asignatura_nombre: Option<String>,
    clase_numero: Option<u32>,
}

/// Convierte texto a kebab-case estricto (quita acentos, signos de puntuación, etc.)
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

/// Parsea el archivo de asistencias y valida formato
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
        let linea_trim = linea.trim();
        
        // Línea vacía
        if linea_trim.is_empty() {
            continue;
        }
        
        // Cabecera
        if en_cabeceras && linea_trim.starts_with('#') {
            // Separador de cabeceras
            if linea_trim.starts_with("# ====") {
                en_cabeceras = false;
                
                // Validar que log sea obligatorio
                if !log_encontrado {
                    return Err("Error: Cabecera '# log: asistencias' es obligatoria".to_string());
                }
                
                continue;
            }
            
            // Parsear cabeceras
			if let Some(valor) = linea_trim.strip_prefix("# log:") {
				let valor_trim = valor.trim();
				if valor_trim != "asistencias" {
					return Err(format!("Línea {}: Cabecera 'log' debe ser 'asistencias', encontrado '{}'", linea_numero, valor_trim));
				}
				log_encontrado = true;
			} else if let Some(valor) = linea_trim.strip_prefix("# curso:") {
				let valor_trim = valor.trim();
				if !valor_trim.is_empty() && valor_trim != "null" {
					metadata.curso_nombre = Some(valor_trim.to_string());
				}
			} else if let Some(valor) = linea_trim.strip_prefix("# asignatura:") {
				let valor_trim = valor.trim();
				if !valor_trim.is_empty() && valor_trim != "null" {
					metadata.asignatura_nombre = Some(valor_trim.to_string());
				}
			} else if let Some(valor) = linea_trim.strip_prefix("# clase:") {
				let valor_trim = valor.trim();
				if !valor_trim.is_empty() && valor_trim != "null" {
					metadata.clase_numero = Some(valor_trim.parse()
						.map_err(|_| format!("Línea {}: Número de clase inválido: '{}'", linea_numero, valor_trim))?);
				}
			} else if linea_trim.starts_with("# fecha_creacion:") {
				// Validar formato YYYYMMDDTHHMMSS±Z
				if let Some(valor) = linea_trim.strip_prefix("# fecha_creacion:") {
					let valor_trim = valor.trim();
					if !valor_trim.is_empty() && valor_trim != "null" {
						// Validación básica del formato
						let regex = regex::Regex::new(r"^\d{8}T\d{6}[+-]\d{4}$").unwrap();
						if !regex.is_match(valor_trim) {
							return Err(format!("Línea {}: Formato de fecha_creacion inválido. Debe ser YYYYMMDDTHHMMSS±Z (ej: 20260702T193230-0500)", linea_numero));
						}
					}
				}
			}
            
            continue;
        }
        
        // Línea de asistencia
        if en_cabeceras {
            return Err(format!("Línea {}: Se esperaba separador de cabeceras '# ====' antes de las líneas de asistencia", linea_numero));
        }
        
        // Validar formato: [XxSs] - <texto>
        let regex = regex::Regex::new(r"^[XxSs]\s+-\s+.+$").unwrap();
        if !regex.is_match(linea_trim) {
            return Err(format!("Línea {}: Formato inválido. Debe ser '[X|S] - <nombre>'. Encontrado: '{}'", linea_numero, linea_trim));
        }
    }
    
    Ok(metadata)
}

/// Determina el ID del curso
fn determinar_curso(ruta_base: &Path, metadata: &MetadataAsistencias, rl: &mut Option<&mut DefaultEditor>, bloque: bool) -> Result<u32, String> {
    let ruta_cursos = ruta_base.join("datos/cursos");
    
    if !ruta_cursos.exists() {
        return Err("No existe el directorio de cursos".to_string());
    }
    
    // Listar cursos disponibles
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
    
    // Si hay nombre de curso en metadata, buscar coincidencia
    if let Some(nombre_curso) = &metadata.curso_nombre {
        let nombre_curso_kebab = a_kebab_case_estricto(nombre_curso);
        if let Some((id, _, _)) = cursos.iter().find(|(_, _, nombre)| {
            let nombre_kebab = a_kebab_case_estricto(nombre);
            nombre_kebab == nombre_curso_kebab
        }) {
            return Ok(*id);
        }
    }
    
    // Modo bloque: si no hay metadata de curso, error
    if bloque {
        return Err("Modo bloque: no se pudo determinar el curso automáticamente. "
                   .to_owned() + "Use la cabecera '# curso:' en los archivos.");
    }
    
    // Modo interactivo (solo se llega aquí si bloque=false, así que rl es Some)
    let rl = rl.as_mut().unwrap();
    
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

/// Determina la información de la asignatura (ID y nombre kebab-case)
fn determinar_asignatura(
    ruta_base: &Path,
    curso_id: &u32,
    metadata: &MetadataAsistencias,
    rl: &mut Option<&mut DefaultEditor>,
    bloque: bool,
) -> Result<Option<(u32, String)>, String> {
    let ruta_cursos = ruta_base.join("datos/cursos");
    
    // Buscar carpeta del curso
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
    
    // Si hay nombre de asignatura en metadata
    if let Some(nombre_asignatura) = &metadata.asignatura_nombre {
        // Intentar extraer ID desde el inicio del nombre (ej: "002 Metodología..." → ID 2)
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
        
        // Si existe el directorio de asignaturas, buscar coincidencia
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
            
            // Prioridad 1: Si se extrajo un ID del nombre, buscar ese ID específico
            if let Some(id) = id_extraido {
                if let Some((_, _, _)) = asignaturas.iter().find(|(asig_id, _, _)| *asig_id == id) {
                    return Ok(Some((id, nombre_kebab)));
                }
                return Ok(Some((id, nombre_kebab)));
            }
            
            // Prioridad 2: Buscar coincidencia por nombre kebab-case
            if let Some((id, _, _)) = asignaturas.iter().find(|(_, _, nombre)| {
                let nombre_kebab_existente = a_kebab_case_estricto(nombre);
                nombre_kebab_existente == nombre_kebab
            }) {
                return Ok(Some((*id, nombre_kebab)));
            }
        }
        
        // Si se extrajo ID del nombre, usarlo directamente sin preguntar
        if let Some(id) = id_extraido {
            if !bloque {
                println!("\nAsignatura detectada: {}", nombre_asignatura);
                println!("ID extraído: {}", id);
                println!("Nombre normalizado: {}", nombre_kebab);
            }
            return Ok(Some((id, nombre_kebab)));
        }
        
        // Modo bloque: si hay asignatura en metadata pero no se pudo determinar ID, error
        if bloque {
            return Err("Modo bloque: no se pudo determinar el ID de la asignatura automáticamente. "
                       .to_owned() + "Incluya el ID al inicio del nombre (ej: '# asignatura: 1 Matematicas')");
        }
        
        // Modo interactivo: preguntar el número (solo se llega aquí si bloque=false)
        let rl = rl.as_mut().unwrap();
        
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
    
    // Si no hay asignatura en metadata, retornar None
    Ok(None)
}

/// Determina el número de clase
fn determinar_clase(metadata: &MetadataAsistencias, archivo_fuente: &Path, rl: &mut Option<&mut DefaultEditor>, bloque: bool) -> Result<u32, String> {
    // Prioridad 1: Cabecera
    if let Some(clase) = metadata.clase_numero {
        return Ok(clase);
    }
    
    // Prioridad 2: Nombre del archivo (buscar patrón cNNN)
    if let Some(nombre) = archivo_fuente.file_stem() {
        let nombre_str = nombre.to_string_lossy();
        let regex = Regex::new(r"c(\d{3})").unwrap();
        
        if let Some(captures) = regex.captures(&nombre_str) {
            if let Some(num_str) = captures.get(1) {
                return Ok(num_str.as_str().parse().unwrap());
            }
        }
    }
    
    // Modo bloque: si no se pudo determinar la clase, error
    if bloque {
        return Err("Modo bloque: no se pudo determinar el número de clase. "
                   .to_owned() + "Use la cabecera '# clase:' o nombre de archivo con patrón cNNN (ej: asistencias-c036.txt)");
    }
    
    // Prioridad 3: Modo interactivo (solo se llega aquí si bloque=false)
    let rl = rl.as_mut().unwrap();
    
    let input = rl.readline("Número de clase: ")
        .map_err(|e| format!("Error al leer entrada: {}", e))?;
    
    input.trim().parse()
        .map_err(|_| "Número de clase inválido".to_string())
}