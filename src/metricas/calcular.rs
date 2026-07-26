use serde_json::json;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use rustyline::DefaultEditor;

/// Modos de visualización para métricas
#[derive(Clone, Debug, PartialEq)]
pub enum ModoMetricas {
    /// Vista en tabla (asignatura, clase, asiste)
    Tabla,
    /// Vista en lista resumida
    Lista,
}

impl std::fmt::Display for ModoMetricas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModoMetricas::Tabla => write!(f, "tabla"),
            ModoMetricas::Lista => write!(f, "lista"),
        }
    }
}

/// Calcula y muestra métricas de asistencias para un curso
pub fn ejecutar(
    ruta_base: &Path,
    tipo_str: &str,
    cursante_filtro: Option<&str>,
    modo_str: &str,
    actualizar: bool,
) -> Result<(), String> {
    if tipo_str != "asistencias" {
        return Err(format!("Cálculo de '{}' aún no implementado. Solo 'asistencias' está disponible.", tipo_str));
    }
    
    let ruta_cursos = ruta_base.join("datos/cursos");
    
    if !ruta_cursos.exists() {
        return Err("No existe el directorio datos/cursos/".to_string());
    }
    
    // Obtener lista de cursos
    let cursos = obtener_cursos(&ruta_cursos)?;
    
    if cursos.is_empty() {
        println!("ℹ No hay cursos registrados.");
        return Ok(());
    }
    
    // Seleccionar curso
    let nombre_curso_seleccionado = if cursos.len() == 1 {
        cursos[0].clone()
    } else {
        seleccionar_curso(&cursos)?
    };
    
    // Calcular métricas de asistencias
    let resultado = calcular_asistencias(&ruta_cursos, &nombre_curso_seleccionado)?;
    
    // Convertir modo string a enum interno
    let modo_enum = match modo_str {
        "tabla" => ModoMetricas::Tabla,
        _ => ModoMetricas::Lista,
    };
    
    // Preparar datos filtrados para mostrar (sin mover el resultado original)
    let cursantes_filtrados: HashMap<String, CursanteMetricas> = if let Some(nombre) = cursante_filtro {
        let mut filtrado = HashMap::new();
        for (k, v) in &resultado.cursantes {
            if k.to_lowercase() == nombre.to_lowercase() || k.contains(nombre) {
                filtrado.insert(k.clone(), v.clone());
            }
        }
        filtrado
    } else {
        resultado.cursantes.clone()
    };
    
    let resultado_filtrado = ResultadoAsistencias {
        nombre_curso: resultado.nombre_curso.clone(),
        total_clases: cursantes_filtrados.values().map(|c| c.asistencias.len() as u32).sum(),
        clases_por_asignatura: HashMap::new(),
        cursantes: cursantes_filtrados,
    };
    
    // Mostrar según el modo
    match modo_enum {
        ModoMetricas::Tabla => mostrar_tabla(&resultado_filtrado),
        ModoMetricas::Lista => mostrar_lista(&resultado_filtrado),
    }
    
    // Actualizar archivo JSON si se solicita (usar datos originales sin filtro)
    if actualizar {
        guardar_json(ruta_base, &nombre_curso_seleccionado, &resultado, &modo_enum)?;
    }
    
    Ok(())
}

/// Registro de asistencia individual
#[derive(Debug, Clone)]
struct Asistencia {
    asignatura: String,
    clase: u32,
    presente: bool,
}

impl Asistencia {
    fn to_json_value(&self) -> serde_json::Value {
        json!({
            "asignatura": self.asignatura,
            "clase": self.clase,
            "presente": self.presente
        })
    }
}

/// Información de un cursante con sus asistencias
#[derive(Debug, Default, Clone)]
struct CursanteMetricas {
    nombre: String,
    asistencias: Vec<Asistencia>,
}

impl CursanteMetricas {
    fn to_json_value(&self) -> serde_json::Value {
        let detalle: Vec<serde_json::Value> = self.asistencias.iter().map(|a| a.to_json_value()).collect();
        
        json!({
            "nombre": self.nombre,
            "presente": self.asistencias.iter().filter(|a| a.presente).count(),
            "ausente": self.asistencias.iter().filter(|a| !a.presente).count(),
            "detalle": detalle
        })
    }
}

/// Resultado del cálculo de asistencias
struct ResultadoAsistencias {
    nombre_curso: String,
    total_clases: u32,
    clases_por_asignatura: HashMap<String, u32>,
    cursantes: HashMap<String, CursanteMetricas>,
}

fn obtener_cursos(ruta_cursos: &Path) -> Result<Vec<String>, String> {
    let mut cursos: Vec<String> = Vec::new();
    
    let entradas = std::fs::read_dir(ruta_cursos)
        .map_err(|e| format!("Error al leer directorio de cursos: {}", e))?;
    
    for entrada in entradas {
        let entrada = entrada.map_err(|e| format!("Error al leer entrada: {}", e))?;
        let ruta = entrada.path();
        
        if ruta.is_dir() {
            if let Some(nombre) = ruta.file_name() {
                let nombre_str = nombre.to_string_lossy().to_string();
                if let Some(id_str) = nombre_str.split('-').next() {
                    if id_str.parse::<u32>().is_ok() {
                        cursos.push(nombre_str);
                    }
                }
            }
        }
    }
    
    cursos.sort();
    Ok(cursos)
}

fn seleccionar_curso(cursos: &[String]) -> Result<String, String> {
    println!("Cursos disponibles:");
    for curso in cursos {
        println!("  {}", curso);
    }
    
    let mut rl = DefaultEditor::new()
        .map_err(|e| format!("Error al inicializar editor: {}", e))?;
    
    let input = rl.readline("\nSeleccione un curso (ID o nombre completo): ")
        .map_err(|e| format!("Error al leer entrada: {}", e))?;
    
    let input = input.trim();
    
    // Intentar por ID
    if let Ok(id) = input.parse::<u32>() {
        for curso in cursos {
            if let Some(curso_id) = curso.split('-').next() {
                if let Ok(curso_id_num) = curso_id.parse::<u32>() {
                    if curso_id_num == id {
                        return Ok(curso.clone());
                    }
                }
            }
        }
        return Err(format!("No se encontró curso con ID {}", id));
    }
    
    // Intentar por nombre
    if let Some(curso) = cursos.iter().find(|c| *c == input) {
        return Ok(curso.clone());
    }
    
    Err(format!("No se encontró curso con nombre '{}'", input))
}

fn calcular_asistencias(ruta_cursos: &Path, nombre_curso: &str) -> Result<ResultadoAsistencias, String> {
    let ruta_curso = ruta_cursos.join(nombre_curso);
    let ruta_asistencias = ruta_curso.join("archivo/asistencias");
    
    let mut total_cursantes: HashMap<String, CursanteMetricas> = HashMap::new();
    let mut total_clases: u32 = 0;
    let mut clases_por_asignatura: HashMap<String, u32> = HashMap::new();
    
    if ruta_asistencias.exists() {
        procesar_directorio_asistencias(&ruta_asistencias, &mut total_cursantes, &mut total_clases, &mut clases_por_asignatura, "")?;
    }
    
    Ok(ResultadoAsistencias {
        nombre_curso: nombre_curso.to_string(),
        total_clases,
        clases_por_asignatura,
        cursantes: total_cursantes,
    })
}

fn procesar_directorio_asistencias(
    ruta: &Path,
    total_cursantes: &mut HashMap<String, CursanteMetricas>,
    total_clases: &mut u32,
    clases_por_asignatura: &mut HashMap<String, u32>,
    asignatura_actual: &str,
) -> Result<(), String> {
    let entradas = std::fs::read_dir(ruta)
        .map_err(|e| format!("Error al leer directorio {}: {}", ruta.display(), e))?;
    
    for entrada in entradas {
        let entrada = entrada.map_err(|e| format!("Error al leer entrada: {}", e))?;
        let ruta_item = entrada.path();
        
        if ruta_item.is_dir() {
            if let Some(nombre) = ruta_item.file_name() {
                let nombre_str = nombre.to_string_lossy().to_string();
                // Extraer el nombre de la asignatura sin el ID
                let asig_nombre = nombre_str.splitn(2, '-').nth(1).unwrap_or("").to_string();
                procesar_directorio_asistencias(&ruta_item, total_cursantes, total_clases, clases_por_asignatura, &asig_nombre)?;
            }
        } else if ruta_item.is_file() {
            if let Some(ext) = ruta_item.extension() {
                if ext == "txt" {
                    *total_clases += 1;
                    clases_por_asignatura.entry(asignatura_actual.to_string()).or_insert(0);
                    *clases_por_asignatura.get_mut(asignatura_actual).unwrap() += 1;
                    
                    // Extraer número de clase del nombre del archivo (clase-NNN.txt)
                    let clase_num = extraer_numero_clase(&ruta_item);
                    
                    procesar_archivo_asistencia(&ruta_item, total_cursantes, asignatura_actual, clase_num)?;
                }
            }
        }
    }
    
    Ok(())
}

fn extraer_numero_clase(ruta: &Path) -> u32 {
    if let Some(nombre) = ruta.file_stem() {
        let nombre_str = nombre.to_string_lossy();
        // Patrón: clase-NNN o cNNN
        let re = regex::Regex::new(r"clase-(\d+)|c(\d+)").unwrap();
        if let Some(caps) = re.captures(&nombre_str) {
            if let Some(m) = caps.get(1) {
                return m.as_str().parse().unwrap_or(0);
            }
            if let Some(m) = caps.get(2) {
                return m.as_str().parse().unwrap_or(0);
            }
        }
    }
    0
}

fn procesar_archivo_asistencia(
    ruta_archivo: &Path,
    total_cursantes: &mut HashMap<String, CursanteMetricas>,
    asignatura: &str,
    clase: u32,
) -> Result<(), String> {
    let contenido = std::fs::read_to_string(ruta_archivo)
        .map_err(|e| format!("Error al leer archivo {}: {}", ruta_archivo.display(), e))?;
    
    for linea in contenido.lines() {
        let linea_limpia = linea.trim();
        
        if linea_limpia.is_empty() || linea_limpia.starts_with('#') {
            continue;
        }
        
        let regex = regex::Regex::new(r"^([XxSs])\s+-\s+(.+)$").unwrap();
        
        if let Some(capturas) = regex.captures(linea_limpia) {
            let estado = capturas.get(1).unwrap().as_str().to_uppercase();
            let nombre = capturas.get(2).unwrap().as_str().trim().to_string();
            
            let metrica = total_cursantes.entry(nombre.clone()).or_insert(CursanteMetricas {
                nombre: nombre.clone(),
                ..Default::default()
            });
            
            metrica.asistencias.push(Asistencia {
                asignatura: asignatura.to_string(),
                clase,
                presente: estado == "S",
            });
        }
    }
    
    Ok(())
}

/// Envía texto a través del paginador (less -F) si está disponible,
/// o lo imprime directamente si no.
fn paginar(salida: &str) {
    // Intentar usar less -F (que automáticamente no pagina si cabe en pantalla)
    let mut child = match std::process::Command::new("less")
        .args(["-F", "-R", "-X"])
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => {
            // Si no hay less, imprimir directamente
            print!("{}", salida);
            return;
        }
    };
    
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(salida.as_bytes());
        // Cerramos stdin para que less sepa que no hay más datos
        drop(stdin);
    }
    
    let _ = child.wait();
}

fn mostrar_lista(resultado: &ResultadoAsistencias) {
    println!();
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  Métricas de Asistencias - {}", resultado.nombre_curso);
    println!("═══════════════════════════════════════════════════════════════════");
    
    if resultado.total_clases == 0 {
        println!("ℹ No hay archivos de asistencias registrados.");
        return;
    }
    
    println!("\nTotal de clases registradas: {}", resultado.total_clases);
    
    if !resultado.clases_por_asignatura.is_empty() {
        println!("\nClases por asignatura:");
        for (asig, count) in &resultado.clases_por_asignatura {
            if asig.is_empty() {
                println!("  Eventos académicos: {}", count);
            } else {
                println!("  {}: {}", asig, count);
            }
        }
    }
    
    println!("\n───────────────────────────────────────────────────────────────");
    println!("  Detalle por Cursante ({}):", resultado.cursantes.len());
    println!("───────────────────────────────────────────────────────────────");
    
    let mut cursantes_ordenados: Vec<_> = resultado.cursantes.iter().collect();
    cursantes_ordenados.sort_by(|a, b| a.1.nombre.cmp(&b.1.nombre));
    
    for (_nombre, metricas) in cursantes_ordenados {
        let presente = metricas.asistencias.iter().filter(|a| a.presente).count();
        let ausente = metricas.asistencias.iter().filter(|a| !a.presente).count();
        
        let porcentaje = if resultado.total_clases > 0 {
            (presente as f64 / resultado.total_clases as f64 * 100.0).round()
        } else {
            0.0
        };
        
        println!("\n  ▸ {}:", metricas.nombre);
        println!("      Presente: {} | Ausente: {} | % Asistencia: {}%", 
                 presente, ausente, porcentaje);
    }
    
    println!("\n═══════════════════════════════════════════════════════════════════");
}

fn mostrar_tabla(resultado: &ResultadoAsistencias) {
    let mut salida = String::new();
    
    salida.push_str("\n═══════════════════════════════════════════════════════════════════\n");
    salida.push_str(&format!("  Tablas de Asistencias - {}\n", resultado.nombre_curso));
    salida.push_str("═══════════════════════════════════════════════════════════════════\n");
    
    if resultado.total_clases == 0 {
        salida.push_str("ℹ No hay datos de asistencia registrados.\n");
        paginar(&salida);
        return;
    }
    
    let ancho_interior = 52;
    
    let mut cursantes_ordenados: Vec<_> = resultado.cursantes.iter().collect();
    cursantes_ordenados.sort_by(|a, b| a.1.nombre.cmp(&b.1.nombre));
    
    for (_nombre, metricas) in cursantes_ordenados {
        let presente = metricas.asistencias.iter().filter(|a| a.presente).count();
        let ausente = metricas.asistencias.iter().filter(|a| !a.presente).count();
        
        let mut todas_ordenadas = metricas.asistencias.clone();
        todas_ordenadas.sort_by(|a, b| {
            let id_a = a.asignatura.split('-').next().and_then(|s| s.parse::<u32>().ok());
            let id_b = b.asignatura.split('-').next().and_then(|s| s.parse::<u32>().ok());
            match (id_a, id_b) {
                (Some(na), Some(nb)) => na.cmp(&nb),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.asignatura.cmp(&b.asignatura),
            }
            .then_with(|| a.clase.cmp(&b.clase))
        });
        
        salida.push('\n');
        salida.push_str(&format!("┌{}┐\n", "─".repeat(ancho_interior)));
        salida.push_str(&format!("│ Cursante: {}\n", metricas.nombre));
        salida.push_str(&format!("│ Total: {}, Si: {}, No: {}\n", metricas.asistencias.len(), presente, ausente));
        salida.push_str(&format!("├{}┤\n", "─".repeat(ancho_interior)));
        salida.push_str(&format!("│ Asignatura{}Clase{}Asiste\n", " ".repeat(18), " ".repeat(6)));
        for asistencia in &todas_ordenadas {
            let estado = if asistencia.presente { "si" } else { "no" };
            let asig_mostrar = if asistencia.asignatura.is_empty() {
                "eventos-academicos".to_string()
            } else {
                asistencia.asignatura.clone()
            };
            
            let asig_trunc = if asig_mostrar.len() > 28 {
                format!("{}...", &asig_mostrar[..25])
            } else {
                asig_mostrar
            };
            
            salida.push_str(&format!("│ {:<28}│{:>10}│{:>8}│\n", asig_trunc, asistencia.clase, estado));
        }
        salida.push_str(&format!("└{}┘\n", "─".repeat(ancho_interior)));
    }
    
    salida.push_str(&format!("\n═══════════════════════════════════════════════════════════════════\n"));
    
    paginar(&salida);
}

fn guardar_json(
    ruta_base: &Path,
    nombre_curso: &str,
    resultado: &ResultadoAsistencias,
    modo: &ModoMetricas,
) -> Result<(), String> {
    let ruta_curso = ruta_base.join("datos/cursos").join(nombre_curso);
    let ruta_metricas = ruta_curso.join("metricas");
    
    // Crear directorio si no existe
    if !ruta_metricas.exists() {
        std::fs::create_dir_all(&ruta_metricas)
            .map_err(|e| format!("Error al crear directorio {}: {}", ruta_metricas.display(), e))?;
    }
    
    // Nombre del archivo según modo
    let archivo = match modo {
        ModoMetricas::Tabla => "asistencias-tabla.json",
        ModoMetricas::Lista => "asistencias-resumen.json",
    };
    
    let ruta_archivo = ruta_metricas.join(archivo);
    
    // Convertir HashMap de cursantes a JSON
    let cursantes_json: HashMap<String, serde_json::Value> = resultado
        .cursantes
        .iter()
        .map(|(k, v)| (k.clone(), v.to_json_value()))
        .collect();
    
    // Construir JSON
    let json_data = json!({
        "curso": nombre_curso,
        "totalClases": resultado.total_clases,
        "clasesPorAsignatura": resultado.clases_por_asignatura,
        "cursantes": cursantes_json,
        "fechaActualizacion": chrono::Local::now().format("%Y%m%dT%H%M%S%z").to_string()
    });
    
    let json_string = serde_json::to_string_pretty(&json_data)
        .map_err(|e| format!("Error al serializar JSON: {}", e))?;
    
    std::fs::write(&ruta_archivo, json_string)
        .map_err(|e| format!("Error al escribir archivo {}: {}", ruta_archivo.display(), e))?;
    
    println!("\n✓ Métricas guardadas en: {}", ruta_archivo.display());
    
    Ok(())
}

/// Muestra métricas guardadas leyendo el JSON
pub fn mostrar(ruta_base: &Path, tipo_str: &str, modo_str: &str) -> Result<(), String> {
    if tipo_str != "asistencias" {
        return Err(format!("Visualización de '{}' aún no implementada. Solo 'asistencias' está disponible.", tipo_str));
    }
    
    let ruta_cursos = ruta_base.join("datos/cursos");
    
    if !ruta_cursos.exists() {
        return Err("No existe el directorio datos/cursos/".to_string());
    }
    
    // Obtener lista de cursos
    let cursos = obtener_cursos(&ruta_cursos)?;
    
    if cursos.is_empty() {
        println!("ℹ No hay cursos registrados.");
        return Ok(());
    }
    
    // Seleccionar curso
    let nombre_curso = if cursos.len() == 1 {
        cursos[0].clone()
    } else {
        seleccionar_curso(&cursos)?
    };
    
    // Buscar archivos de métricas
    let ruta_metricas = ruta_cursos.join(&nombre_curso).join("metricas");
    
    if !ruta_metricas.exists() {
        println!("ℹ No hay métricas guardadas para el curso '{}'.", nombre_curso);
        println!("Use 'trazar metricas calcular -t asistencias -a' para generar métricas.");
        return Ok(());
    }
    
    // Buscar archivo según modo
    let mut ruta_archivo = match modo_str {
        "tabla" => ruta_metricas.join("asistencias-tabla.json"),
        _ => ruta_metricas.join("asistencias-resumen.json"),
    };
    
    if !ruta_archivo.exists() {
        // Fallback al otro archivo si el específico no existe
        let fallback = if modo_str == "tabla" {
            ruta_metricas.join("asistencias-resumen.json")
        } else {
            ruta_metricas.join("asistencias-tabla.json")
        };
        
        if fallback.exists() {
            println!("ℹ Archivo '{}' no encontrado. Usando '{}' en su lugar.",
                     ruta_archivo.file_name().unwrap_or_default().to_string_lossy(),
                     fallback.file_name().unwrap_or_default().to_string_lossy());
            ruta_archivo = fallback;
        } else {
            return Err("No se encontró archivo de métricas. Ejecute 'trazar metricas calcular -t asistencias -a' primero.".to_string());
        }
    }
    
    // Leer y parsear JSON
    let contenido = std::fs::read_to_string(&ruta_archivo)
        .map_err(|e| format!("Error al leer archivo {}: {}", ruta_archivo.display(), e))?;
    
    let datos: serde_json::Value = serde_json::from_str(&contenido)
        .map_err(|e| format!("Error al parsear JSON: {}", e))?;
    
    // Mostrar datos según modo
    match modo_str {
        "tabla" => mostrar_json_tabla(&datos, &nombre_curso)?,
        _ => mostrar_json_lista(&datos, &nombre_curso)?,
    }
    
    Ok(())
}

fn mostrar_json_lista(datos: &serde_json::Value, nombre_curso: &str) -> Result<(), String> {
    println!();
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  Métricas Guardadas - {}", nombre_curso);
    println!("═══════════════════════════════════════════════════════════════════");
    
    // Mostrar información general
    if let Some(total) = datos.get("totalClases").and_then(|v| v.as_u64()) {
        println!("\nTotal de clases: {}", total);
    }
    
    if let Some(fecha) = datos.get("fechaActualizacion").and_then(|v| v.as_str()) {
        println!("Fecha de actualización: {}", fecha);
    }
    
    // Mostrar cursantes
    if let Some(cursantes) = datos.get("cursantes").and_then(|v| v.as_object()) {
        println!("\n───────────────────────────────────────────────────────────────");
        println!("  Cursantes ({}):", cursantes.len());
        println!("───────────────────────────────────────────────────────────────");
        
        let mut nombres: Vec<_> = cursantes.keys().collect();
        nombres.sort();
        
        for nombre in nombres {
            if let Some(info) = cursantes.get(nombre) {
                let presente = info.get("presente").and_then(|v| v.as_u64()).unwrap_or(0);
                let ausente = info.get("ausente").and_then(|v| v.as_u64()).unwrap_or(0);
                let total = presente + ausente;
                let porcentaje = if total > 0 {
                    (presente as f64 / total as f64 * 100.0).round()
                } else {
                    0.0
                };
                
                println!("\n  ▸ {}:", nombre);
                println!("      Presente: {} | Ausente: {} | % Asistencia: {}%", 
                         presente, ausente, porcentaje);
                
                // Mostrar detalle si existe
                if let Some(detalle) = info.get("detalle").and_then(|v| v.as_array()) {
                    if !detalle.is_empty() {
                        println!("      Detalle de asistencias:");
                        for asis in detalle {
                            let asig = asis.get("asignatura").and_then(|v| v.as_str()).unwrap_or("N/A");
                            let clase = asis.get("clase").and_then(|v| v.as_u64()).unwrap_or(0);
                            let presente = asis.get("presente").and_then(|v| v.as_bool()).unwrap_or(false);
                            let estado = if presente { "si" } else { "no" };
                            println!("        - {} (clase {}): {}", asig, clase, estado);
                        }
                    }
                }
            }
        }
    }
    
    println!("\n═══════════════════════════════════════════════════════════════════");
    
    Ok(())
}

fn mostrar_json_tabla(datos: &serde_json::Value, nombre_curso: &str) -> Result<(), String> {
    let ancho_interior = 52;
    let mut salida = String::new();
    
    salida.push('\n');
    salida.push_str("═══════════════════════════════════════════════════════════════════\n");
    salida.push_str(&format!("  Tablas de Asistencias - {}\n", nombre_curso));
    salida.push_str("═══════════════════════════════════════════════════════════════════\n");
    
    if let Some(total) = datos.get("totalClases").and_then(|v| v.as_u64()) {
        salida.push_str(&format!("\nTotal de clases: {}\n", total));
    }
    
    if let Some(fecha) = datos.get("fechaActualizacion").and_then(|v| v.as_str()) {
        salida.push_str(&format!("Fecha de actualización: {}\n", fecha));
    }
    
    // Mostrar cursantes en tabla
    if let Some(cursantes) = datos.get("cursantes").and_then(|v| v.as_object()) {
        let mut nombres: Vec<_> = cursantes.keys().collect();
        nombres.sort();
        
        for nombre in nombres {
            if let Some(info) = cursantes.get(nombre) {
                let presente = info.get("presente").and_then(|v| v.as_u64()).unwrap_or(0);
                let ausente = info.get("ausente").and_then(|v| v.as_u64()).unwrap_or(0);
                
                salida.push('\n');
                salida.push_str(&format!("┌{}┐\n", "─".repeat(ancho_interior)));
                salida.push_str(&format!("│ Cursante: {}\n", nombre));
                salida.push_str(&format!("│ Total: {}, Si: {}, No: {}\n", presente + ausente, presente, ausente));
                salida.push_str(&format!("├{}┤\n", "─".repeat(ancho_interior)));
                salida.push_str(&format!("│ Asignatura{}Clase{}Asiste\n", " ".repeat(18), " ".repeat(6)));
                
                // Mostrar detalle si existe
                if let Some(detalle) = info.get("detalle").and_then(|v| v.as_array()) {
                    if !detalle.is_empty() {
                        // Ordenar detalle por asignatura y clase
                        let mut detalle_ordenado: Vec<&serde_json::Value> = detalle.iter().collect();
                        detalle_ordenado.sort_by(|a, b| {
                            let asig_a = a.get("asignatura").and_then(|v| v.as_str()).unwrap_or("");
                            let asig_b = b.get("asignatura").and_then(|v| v.as_str()).unwrap_or("");
                            let id_a = asig_a.split('-').next().and_then(|s| s.parse::<u32>().ok());
                            let id_b = asig_b.split('-').next().and_then(|s| s.parse::<u32>().ok());
                            match (id_a, id_b) {
                                (Some(na), Some(nb)) => na.cmp(&nb),
                                (Some(_), None) => std::cmp::Ordering::Less,
                                (None, Some(_)) => std::cmp::Ordering::Greater,
                                (None, None) => asig_a.cmp(asig_b),
                            }
                            .then_with(|| {
                                let clase_a = a.get("clase").and_then(|v| v.as_u64()).unwrap_or(0);
                                let clase_b = b.get("clase").and_then(|v| v.as_u64()).unwrap_or(0);
                                clase_a.cmp(&clase_b)
                            })
                        });
                        
                        for asis in detalle_ordenado {
                            let asig = asis.get("asignatura").and_then(|v| v.as_str()).unwrap_or("");
                            let clase = asis.get("clase").and_then(|v| v.as_u64()).unwrap_or(0);
                            let presente = asis.get("presente").and_then(|v| v.as_bool()).unwrap_or(false);
                            let estado = if presente { "si" } else { "no" };
                            
                            let asig_mostrar = if asig.is_empty() {
                                "eventos-academicos".to_string()
                            } else {
                                asig.to_string()
                            };
                            
                            let asig_trunc = if asig_mostrar.len() > 28 {
                                format!("{}...", &asig_mostrar[..25])
                            } else {
                                asig_mostrar
                            };
                            
                            salida.push_str(&format!("│ {:<28}│{:>10}│{:>8}│\n", asig_trunc, clase, estado));
                        }
                    }
                }
                salida.push_str(&format!("└{}┘\n", "─".repeat(ancho_interior)));
            }
        }
    }
    
    salida.push_str(&format!("\n═══════════════════════════════════════════════════════════════════\n"));
    
    paginar(&salida);
    
    Ok(())
}
