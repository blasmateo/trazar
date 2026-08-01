use clap::{Parser, Subcommand, CommandFactory, ValueEnum};
use clap_complete::{generate, Shell};
use std::path::PathBuf;
use colored::Colorize;

mod curso;
mod cursante;
mod archivo;
mod metricas;

/// Tipos de dataset para importación/exportación (con autocompletado)
#[derive(ValueEnum, Clone, Debug)]
enum TipoDatasetCli {
    Asistencias,
    Quizzes,
    Asignaciones,
    Pagos,
}

impl std::fmt::Display for TipoDatasetCli {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TipoDatasetCli::Asistencias => write!(f, "asistencias"),
            TipoDatasetCli::Quizzes => write!(f, "quizzes"),
            TipoDatasetCli::Asignaciones => write!(f, "asignaciones"),
            TipoDatasetCli::Pagos => write!(f, "pagos"),
        }
    }
}

// Templates de ayuda en español con mejor formato - minimalista y claro
const HELP_CMD: &str = "{before-help}{name} {version}
{about-with-newline}

{usage-heading} {usage}

{all-args}

{after-help}";

const HELP_SUBCMD: &str = "{before-help}{about-with-newline}

{usage-heading} {usage}

{all-args}

{after-help}";

#[derive(Parser)]
#[command(
    name = "trazar".bright_blue().to_string(),
    about = "Gestión académica simple y eficiente".green().to_string(),
    version,
    help_template = HELP_CMD,
    override_usage = "trazar <COMANDO>".yellow().to_string(),
    after_help = "Ejemplos:\n  trazar curso nuevo\n  trazar archivo init\n  trazar completions bash\n\nUsa 'trazar <comando> --help' para más ayuda.".dimmed().to_string()
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Gestión de cursos (nuevo, mostrar, editar, remover)
    #[command(
        help_template = HELP_CMD,
        override_usage = "trazar curso <COMANDO>"
    )]
    Curso {
        #[command(subcommand)]
        action: CursoAction,
    },

    /// Gestión de cursantes (nuevo, mostrar, editar, remover)
    #[command(
        help_template = HELP_CMD,
        override_usage = "trazar cursante [OPCIONES] <COMANDO>"
    )]
    Cursante {
        /// Curso al que pertenece el cursante
        #[arg(short = 'c', long = "curso")]
        curso: Option<String>,
        #[command(subcommand)]
        action: CursanteAction,
    },

    /// Gestión de archivos e inspección (init, importar, exportar, mostrar, remover)
    #[command(
        help_template = HELP_CMD,
        override_usage = "trazar archivo [OPCIONES] [COMANDO]",
        long_about = "Gestiona archivos de datos e inspecciona la estructura.\n\nUso común:\n  trazar archivo -i verificar       Verifica integridad de directorios\n  trazar archivo -i validar -t asistencias   Valida archivos importados\n  trazar archivo init               Crea estructura base\n  trazar archivo importar -c 1 -t asistencias -r archivo.txt\n  trazar archivo exportar -t asistencias -m lista -r salida.docx"
    )]
    Archivo {
        /// Inspeccionar: validar o verificar
        #[arg(short = 'i', long = "inspeccionar", value_name = "ACCION")]
        inspeccionar: Option<AccionInspeccion>,
        /// Tipo de dataset (para validar y mostrar)
        #[arg(short = 't', long = "tipo", value_name = "TIPO", requires = "inspeccionar")]
        tipo: Option<TipoDatasetCli>,
        #[command(subcommand)]
        action: Option<ArchivoAction>,
    },

    /// Métricas y reportes (mostrar, calcular)
    #[command(
        help_template = HELP_CMD,
        override_usage = "trazar metricas <COMANDO>"
    )]
    Metricas {
        #[command(subcommand)]
        action: MetricasAction,
    },

    /// Generar autocompletado para tu shell
    #[command(
        help_template = HELP_SUBCMD,
        override_usage = "trazar completions <SHELL> [RUTA]"
    )]
    Completions {
        /// Shell: bash, zsh, fish, powershell
        shell: Shell,
        /// Ruta donde guardar el script
        ruta: Option<String>,
    },
}

#[derive(Subcommand)]
enum CursoAction {
    /// Crear un nuevo curso
    #[command(help_template = HELP_SUBCMD, override_usage = "trazar curso nuevo")]
    Nuevo,
    
    /// Mostrar cursos (lista o ficha específica)
    #[command(help_template = HELP_SUBCMD, override_usage = "trazar curso mostrar [OPCIONES]")]
    Mostrar {
        /// ID o nombre del curso
        #[arg(short = 'i', long = "id")]
        id: Option<String>,
    },
    
    /// Editar un curso existente
    #[command(help_template = HELP_SUBCMD, override_usage = "trazar curso editar [OPCIONES]")]
    Editar {
        /// ID o nombre del curso
        #[arg(short = 'i', long = "id")]
        id: Option<String>,
    },
    
    /// Remover cursos
    #[command(help_template = HELP_SUBCMD, override_usage = "trazar curso remover [OPCIONES]")]
    Remover {
        /// IDs o nombres de cursos a remover (si se omite, modo interactivo)
        #[arg(short = 'i', long = "id", num_args = 0..)]
        ids: Vec<String>,
    },
}

#[derive(Subcommand)]
enum CursanteAction {
    /// Agregar cursante
    #[command(help_template = HELP_SUBCMD, override_usage = "trazar cursante nuevo")]
    Nuevo,
    
    /// Mostrar cursantes (lista o ficha específica)
    #[command(help_template = HELP_SUBCMD, override_usage = "trazar cursante mostrar [OPCIONES]")]
    Mostrar {
        /// ID o nombre de cursante
        #[arg(short = 'i', long = "id")]
        id: Option<String>,
    },
    
    /// Editar cursante
    #[command(help_template = HELP_SUBCMD, override_usage = "trazar cursante editar [OPCIONES]")]
    Editar {
        /// ID o nombre de cursante
        #[arg(short = 'i', long = "id")]
        id: Option<String>,
    },
    
    /// Remover cursantes
    #[command(help_template = HELP_SUBCMD, override_usage = "trazar cursante remover [OPCIONES]")]
    Remover {
        /// IDs o nombres de cursantes a remover (si se omite, modo interactivo)
        #[arg(short = 'i', long = "id", num_args = 0..)]
        ids: Vec<String>,
    },
}

#[derive(Subcommand)]
enum ArchivoAction {
    /// Crear estructura base de directorios
    #[command(help_template = HELP_SUBCMD, override_usage = "trazar archivo init")]
    Init,
    
    /// Purgar toda la base de datos (requiere confirmación)
    #[command(help_template = HELP_SUBCMD, override_usage = "trazar archivo purgar")]
    Purgar,
    
    /// Importar archivos crudos
    #[command(
        help_template = HELP_SUBCMD,
        override_usage = "trazar archivo importar -c <CURSO> -t <TIPO> -r <RUTA>... [-s]",
        long_about = "Importa archivos crudos al directorio del curso.\n\nUso:\n  trazar archivo importar -c 1 -t asistencias -r archivo.txt\n  trazar archivo importar -c matematicas -t asistencias -r *.txt -s\n\nOpciones:\n  -c, --curso     Curso destino (ID o nombre)\n  -t, --tipo      Tipo: asistencias, quizzes, asignaciones, pagos\n  -r, --ruta      Archivos o directorios a importar\n  -s, --si        Auto-afirmar (sin preguntar)"
    )]
    Importar {
        /// Curso destino (ID o nombre)
        #[arg(short = 'c', long = "curso", value_name = "CURSO", required = true)]
        curso: String,
        /// Tipo de dataset
        #[arg(short = 't', long = "tipo", value_name = "TIPO", required = true)]
        tipo: TipoDatasetCli,
        /// Archivos o directorios a importar
        #[arg(short = 'r', long = "ruta", value_name = "RUTA", required = true, num_args = 1..)]
        archivos: Vec<String>,
        /// Auto-afirmar (importar sin preguntar)
        #[arg(short = 's', long = "si")]
        si: bool,
    },

    /// Exportar datos a .docx
    #[command(
        help_template = HELP_SUBCMD,
        override_usage = "trazar archivo exportar -t <TIPO> -m [lista|tabla] -r <RUTA>"
    )]
    Exportar {
        /// Tipo de dataset
        #[arg(short = 't', long = "tipo", value_name = "TIPO")]
        tipo: TipoDatasetCli,
        /// Modo: lista (resumen) o tabla (detallado)
        #[arg(short = 'm', long = "modo", value_name = "MODO", default_value = "lista")]
        modo: ModoMetricas,
        /// Ruta de salida del .docx
        #[arg(short = 'r', long = "ruta", value_name = "RUTA")]
        ruta: String,
    },

    /// Listar archivos
    #[command(
        help_template = HELP_SUBCMD,
        override_usage = "trazar archivo mostrar [TIPO]"
    )]
    Mostrar {
        /// Tipo de dataset (opcional)
        #[arg(value_name = "TIPO")]
        tipo: Option<String>,
    },

    /// Remover archivos
    #[command(
        help_template = HELP_SUBCMD,
        override_usage = "trazar archivo remover [ARCHIVO...]"
    )]
    Remover {
        /// Archivos a remover (interactivo si se omite)
        #[arg(value_name = "ARCHIVO", num_args = 0..)]
        archivos: Vec<String>,
    },
}

/// Acciones de inspección para `trazar archivo inspeccionar`
#[derive(ValueEnum, Clone, Debug)]
enum AccionInspeccion {
    /// Validar formato semántico de archivos
    Validar,
    /// Verificar integridad de directorios
    Verificar,
}

impl std::fmt::Display for AccionInspeccion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccionInspeccion::Validar => write!(f, "validar"),
            AccionInspeccion::Verificar => write!(f, "verificar"),
        }
    }
}

#[derive(Subcommand)]
enum MetricasAction {
    /// Mostrar métricas guardadas
    #[command(help_template = HELP_SUBCMD, override_usage = "trazar metricas mostrar -t <TIPO> [-m <MODO>]")]
    Mostrar {
        /// Tipo de dataset
        #[arg(short = 't', long = "tipo", value_name = "TIPO")]
        tipo: TipoDatasetCli,
        /// Modo: tabla o lista
        #[arg(short = 'm', long = "modo", value_name = "MODO", default_value = "lista")]
        modo: ModoMetricas,
    },
    
    /// Calcular métricas
    #[command(help_template = HELP_SUBCMD, override_usage = "trazar metricas calcular -t <TIPO> [-c <CURSANTE>] [-m <MODO>] [-a]")]
    Calcular {
        /// Tipo de dataset
        #[arg(short = 't', long = "tipo", value_name = "TIPO")]
        tipo: TipoDatasetCli,
        /// Filtrar por cursante
        #[arg(short = 'c', long = "cursante", value_name = "CURSANTE")]
        cursante: Option<String>,
        /// Modo: tabla o lista
        #[arg(short = 'm', long = "modo", value_name = "MODO", default_value = "lista")]
        modo: ModoMetricas,
        /// Actualizar JSON de métricas
        #[arg(short = 'a', long = "actualizar")]
        actualizar: bool,
    },
}

/// Modos de visualización para métricas
#[derive(ValueEnum, Clone, Debug)]
enum ModoMetricas {
    /// Vista en tabla
    Tabla,
    /// Vista en lista
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

fn main() {
    let cli = Cli::parse();
    let ruta_base = obtener_ruta_base();
    
    match cli.command {
        Commands::Curso { action } => {
            match action {
                CursoAction::Nuevo => {
                    match curso::nuevo(&ruta_base) {
                        Ok(_) => {},
                        Err(e) => eprintln!("✗ Error: {}", e),
                    }
                }
                CursoAction::Mostrar { id } => {
                    match curso::mostrar(&ruta_base, id.as_deref()) {
                        Ok(_) => {},
                        Err(e) => eprintln!("✗ Error: {}", e),
                    }
                }
                CursoAction::Editar { id } => {
                    match curso::editar(&ruta_base, id.as_deref()) {
                        Ok(_) => {},
                        Err(e) => eprintln!("✗ Error: {}", e),
                    }
                }
                CursoAction::Remover { ids } => {
                    let nombres = if ids.is_empty() {
                        None
                    } else {
                        Some(ids.as_slice())
                    };
                    match curso::remover(&ruta_base, nombres) {
                        Ok(_) => {},
                        Err(e) => eprintln!("✗ Error: {}", e),
                    }
                }
            }
        }
        
        Commands::Cursante { curso, action } => {
            let curso_arg = curso.as_deref().filter(|s| !s.is_empty());
            
            match action {
                CursanteAction::Nuevo => {
                    match cursante::nuevo(&ruta_base, curso_arg) {
                        Ok(_) => {},
                        Err(e) => eprintln!("✗ Error: {}", e),
                    }
                }
                CursanteAction::Mostrar { id } => {
                    match cursante::mostrar(&ruta_base, curso_arg, id.as_deref()) {
                        Ok(_) => {},
                        Err(e) => eprintln!("✗ Error: {}", e),
                    }
                }
                CursanteAction::Editar { id } => {
                    match cursante::editar(&ruta_base, curso_arg, id.as_deref()) {
                        Ok(_) => {},
                        Err(e) => eprintln!("✗ Error: {}", e),
                    }
                }
                CursanteAction::Remover { ids } => {
                    let nombres = if ids.is_empty() {
                        None
                    } else {
                        Some(ids.as_slice())
                    };
                    match cursante::remover(&ruta_base, curso_arg, nombres) {
                        Ok(_) => {},
                        Err(e) => eprintln!("✗ Error: {}", e),
                    }
                }
            }
        }
		
				Commands::Archivo { inspeccionar, tipo, action, .. } => {
			// Flag -i/--inspeccionar tiene prioridad sobre los subcomandos
			if let Some(accion) = inspeccionar {
				match accion {
					AccionInspeccion::Verificar => {
						match archivo::verificar(&ruta_base) {
							Ok(faltantes) => {
								if faltantes.is_empty() {
									println!("✓ Integridad OK: todos los directorios existen");
								} else {
									println!("⚠ Faltan directorios:");
									for dir in faltantes {
										println!("  - {}", dir);
									}
								}
							}
							Err(e) => eprintln!("✗ Error: {}", e),
						}
					}
					AccionInspeccion::Validar => {
						let tipo_str = tipo.as_ref().map(|t| t.to_string());
						match archivo::validar(&ruta_base, tipo_str.as_deref()) {
							Ok(_) => {},
							Err(e) => eprintln!("✗ Error: {}", e),
						}
					}
				}
				return;
			}
			// Sin flag: delegar a subcomandos
			match action {
				Some(ArchivoAction::Init) => {
					match archivo::init(&ruta_base) {
						Ok(_) => println!("✓ Estructura base creada/verificada"),
						Err(e) => eprintln!("✗ Error: {}", e),
					}
				}
				Some(ArchivoAction::Purgar) => {
					match archivo::purgar(&ruta_base) {
						Ok(_) => {},
						Err(e) => eprintln!("✗ Error: {}", e),
					}
				}
								Some(ArchivoAction::Importar { curso, tipo, archivos, si }) => {
					let curso_str = if curso.trim().is_empty() { None } else { Some(curso.as_str()) };
					let tipo_str = tipo.to_string();
					match archivo::importar(&ruta_base, &tipo_str, &archivos, si, curso_str) {
						Ok(_) => {},
						Err(e) => eprintln!("✗ Error: {}", e),
					}
				}
				Some(ArchivoAction::Exportar { tipo, modo, ruta }) => {
					let tipo_str = tipo.to_string();
					let modo_str = modo.to_string();
					match archivo::exportar(&ruta_base, &tipo_str, &modo_str, &ruta) {
						Ok(_) => {},
						Err(e) => eprintln!("✗ Error: {}", e),
					}
				}
				Some(ArchivoAction::Mostrar { tipo }) => {
					match archivo::mostrar(&ruta_base, tipo.as_deref()) {
						Ok(_) => {},
						Err(e) => eprintln!("✗ Error: {}", e),
					}
				}
				Some(ArchivoAction::Remover { archivos }) => {
					let rutas = if archivos.is_empty() {
						None
					} else {
						Some(archivos.as_slice())
					};
					match archivo::remover(&ruta_base, rutas) {
						Ok(_) => {},
						Err(e) => eprintln!("✗ Error: {}", e),
					}
				}
				None => {
					// Sin flag ni subcomando: imprimir ayuda
					let mut cmd = Cli::command();
					if let Some(sub) = cmd.find_subcommand_mut("archivo") {
						let _ = sub.print_help();
					}
				}
			}
		}
		
		Commands::Metricas { action } => {
			match action {
				MetricasAction::Mostrar { tipo, modo } => {
					let tipo_str = tipo.to_string();
					let modo_str = modo.to_string();
					match metricas::mostrar(&ruta_base, &tipo_str, &modo_str) {
						Ok(_) => {},
						Err(e) => eprintln!("✗ Error: {}", e),
					}
				}
				MetricasAction::Calcular { tipo, cursante, modo, actualizar } => {
					let tipo_str = tipo.to_string();
					let modo_str = modo.to_string();
					let cursante_filtro = cursante.as_deref();
					match metricas::calcular(&ruta_base, &tipo_str, cursante_filtro, &modo_str, actualizar) {
						Ok(_) => {},
						Err(e) => eprintln!("✗ Error: {}", e),
					}
				}
			}
		}
        
        Commands::Completions { shell, ruta } => {
            eprintln!("=== Instrucciones para autocompletado ===\n");
            
            match shell {
                Shell::Fish => {
                    eprintln!("1. Crear el directorio si no existe:");
                    eprintln!("   mkdir -p ~/.config/fish/completions/\n");
                    eprintln!("2. Ejecutar este comando:");
                    eprintln!("   trazar completions fish ~/.config/fish/completions/trazar.fish\n");
                    eprintln!("3. Reiniciar fish:");
                    eprintln!("   exec fish\n");
                    eprintln!("4. Probar:");
                    eprintln!("   trazar archivo <TAB>\n");
                }
                Shell::Bash => {
                    eprintln!("1. Crear el directorio si no existe:");
                    eprintln!("   mkdir -p ~/.bash_completion.d/\n");
                    eprintln!("2. Ejecutar este comando:");
                    eprintln!("   trazar completions bash ~/.bash_completion.d/trazar\n");
                    eprintln!("3. Activar:");
                    eprintln!("   source ~/.bash_completion.d/trazar\n");
                    eprintln!("4. Probar:");
                    eprintln!("   trazar archivo <TAB>\n");
                }
                Shell::Zsh => {
                    eprintln!("1. Crear el directorio si no existe:");
                    eprintln!("   mkdir -p ~/.zsh/completions/\n");
                    eprintln!("2. Ejecutar este comando:");
                    eprintln!("   trazar completions zsh ~/.zsh/completions/_trazar\n");
                    eprintln!("3. Agregar al .zshrc:");
                    eprintln!("   fpath=(~/.zsh/completions $fpath)");
                    eprintln!("   autoload -U compinit && compinit\n");
                    eprintln!("4. Reiniciar zsh:");
                    eprintln!("   exec zsh\n");
                    eprintln!("5. Probar:");
                    eprintln!("   trazar archivo <TAB>\n");
                }
                Shell::PowerShell => {
                    eprintln!("1. Ejecutar este comando:");
                    eprintln!("   trazar completions powershell $PROFILE\n");
                    eprintln!("2. Reiniciar PowerShell\n");
                    eprintln!("3. Probar:");
                    eprintln!("   trazar archivo <TAB>\n");
                }
                _ => {
                    eprintln!("Shell soportado: {}", shell);
                    eprintln!("Usar el comando de generación apropiado para la shell.\n");
                }
            }
            
            eprintln!("=== Fin de instrucciones ===\n");
            
            if ruta.is_none() {
                return;
            }
            
            let ruta_str = ruta.unwrap();
            
            let ruta_expandida = if ruta_str.starts_with("~/") {
                if let Some(home) = std::env::var_os("HOME") {
                    PathBuf::from(home).join(&ruta_str[2..])
                } else {
                    PathBuf::from(ruta_str)
                }
            } else {
                PathBuf::from(ruta_str)
            };
            
            if let Some(parent) = ruta_expandida.parent() {
                if !parent.exists() {
                    match std::fs::create_dir_all(parent) {
                        Ok(_) => eprintln!("✓ Directorio creado: {}", parent.display()),
                        Err(e) => {
                            eprintln!("✗ Error al crear directorio {}: {}", parent.display(), e);
                            return;
                        }
                    }
                }
            }
            
            let mut script_content = Vec::new();
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "trazar", &mut script_content);
            
            match std::fs::write(&ruta_expandida, script_content) {
                Ok(_) => {
                    eprintln!("✓ Script guardado en: {}", ruta_expandida.display());
                    eprintln!("\nRecuerde reiniciar la shell para que los cambios surtan efecto.");
                }
                Err(e) => {
                    eprintln!("✗ Error al guardar el script: {}", e);
                }
            }
        }
    }
}

fn obtener_ruta_base() -> PathBuf {
    std::env::current_exe()
        .expect("No se pudo obtener la ruta del ejecutable")
        .parent()
        .expect("No se pudo obtener el directorio base")
        .to_path_buf()
}