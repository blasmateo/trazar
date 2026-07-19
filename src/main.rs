use clap::{Parser, Subcommand, CommandFactory, ValueEnum};
use clap_complete::{generate, Shell};
use std::path::PathBuf;

mod inspector;
mod curso;
mod cursante;
mod archivo;

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

// Templates de ayuda en español
const HELP_CMD: &str = "\
{about-with-newline}
Uso: {usage}

Comandos:
{subcommands}
Opciones:
{options}
";

const HELP_SUBCMD: &str = "\
{about-with-newline}
Uso: {usage}

Opciones:
{options}
";

#[derive(Parser)]
#[command(
    name = "trazar",
    about = "Trazabilidad Académica y Reporte",
    version,
    help_template = HELP_CMD,
    override_usage = "trazar <COMANDO>"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Gestión de estructura de datos
    #[command(
        help_template = HELP_CMD,
        override_usage = "trazar inspector <COMANDO>"
    )]
    Inspector {
        #[command(subcommand)]
        action: InspectorAction,
    },
    
    /// Gestión de cursos
    #[command(
        help_template = HELP_CMD,
        override_usage = "trazar curso <COMANDO>"
    )]
    Curso {
        #[command(subcommand)]
        action: CursoAction,
    },
    
    /// Gestión de cursantes
    #[command(
        help_template = HELP_CMD,
        override_usage = "trazar cursante [OPCIONES] <COMANDO>"
    )]
    Cursante {
        /// Curso al que pertenece cada cursante
        #[arg(short = 'c', long = "curso")]
        curso: Option<String>,
        #[command(subcommand)]
        action: CursanteAction,
    },

	/// Gestión de archivos de datos
	#[command(
		help_template = HELP_CMD,
		override_usage = "trazar archivo <COMANDO>"
	)]
	Archivo {
		#[command(subcommand)]
		action: ArchivoAction,
	},
    
    /// Generar scripts de autocompletado
    #[command(
        help_template = HELP_SUBCMD,
        override_usage = "trazar completions <SHELL> [RUTA]"
    )]
    Completions {
        /// Shell para el que generar el script
        shell: Shell,
        /// Ruta donde guardar el script
        ruta: Option<String>,
    },
}

#[derive(Subcommand)]
enum InspectorAction {
    /// Iniciar directorios base
    #[command(help_template = HELP_SUBCMD, override_usage = "trazar inspector init")]
    Init,
    /// Verificar integridad de directorios
    #[command(help_template = HELP_SUBCMD, override_usage = "trazar inspector verificar")]
    Verificar,
    /// Purgar toda la base de datos de usuario
    #[command(help_template = HELP_SUBCMD, override_usage = "trazar inspector purgar")]
    Purgar,
    /// Validar archivos en datos/archivo/
	#[command(help_template = HELP_SUBCMD, override_usage = "trazar inspector validar")]
	Validar {
		/// Tipo de dataset (opcional)
        tipo: Option<String>,
    },
    /// Consolidar archivos validados a datos/cursos/
	#[command(help_template = HELP_SUBCMD, override_usage = "trazar inspector consolidar")]
    Consolidar {
        /// ID o nombre del curso
        #[arg(short = 'c', long = "curso")]
        curso: String,
        /// Tipo de dataset (opcional)
        tipo: Option<String>,
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
    /// Importar archivo crudo a datos/cursos/<id-curso>/archivo/
    #[command(
        help_template = HELP_SUBCMD,
        override_usage = "trazar archivo importar -t <TIPO> -r <RUTA>... [OPCIONES]",
        long_about = "Importa uno o más archivos crudos al directorio datos/cursos/<id-curso>/archivo/asistencias/.

TIPOS VÁLIDOS (-t, --tipo):
  asistencias    Archivo de registro de asistencias a clases
  quizzes        Archivo de resultados de quizzes (próximamente)
  asignaciones   Archivo de estados de asignaciones (próximamente)
  pagos          Archivo de registros de pagos (próximamente)

RUTAS VÁLIDAS (-r, --ruta):
  - Archivos individuales:  -r archivo1.txt archivo2.txt
  - Directorios:            -r ./carpeta-con-archivos/
  - Globs (expandidos por el shell):  -r *.txt

OPCIONES:
  -s, --si         Importa solo los archivos válidos sin preguntar (auto-afirmación)

FORMATO DEL ARCHIVO DE ASISTENCIAS:

El archivo debe tener cabeceras seguidas de un separador y las líneas de asistencia:

  # log: asistencias
  # curso: <nombre-del-curso>
  # asignatura: <nombre-de-asignatura>
  # clase: <numero>
  # fecha_creacion: <YYYYMMDDTHHMMSS±Z>
  # ====================
  
  x - Nombre Apellido Uno
  s - Nombre Apellido Dos
  x - Nombre Apellido Tres

CAMPOS DE CABECERA:
  - log:              Obligatorio. Debe ser 'asistencias'
  - curso:            Opcional. Nombre del curso (debe coincidir exactamente con curso registrado)
  - asignatura:       Opcional. Nombre de asignatura (si se omite va a eventos-academicos)
  - clase:            Opcional. Número de clase (también se puede extraer del nombre del archivo con patrón cNNN)
  - fecha_creacion:   Opcional. Timestamp de creación en formato YYYYMMDDTHHMMSS±Z (ej: 20260702T193230-0500)

LÍNEAS DE ASISTENCIA:
  - Formato estricto: [x|s|X|S] - <Nombre Completo> (requiere x/s/S/X)
  - 'x' o 'X': ausente
  - 's' o 'S': presente
  - Se permiten líneas vacías y comentarios con #

NOMBRE DEL ARCHIVO:
  - Debe contener el patrón 'cNNN' (ej: asistencias-c036.txt)
  - El número de clase se extrae de este patrón

EJEMPLOS DE USO:
  # Importar un solo archivo
  trazar archivo importar -t asistencias -r asistencias-c036.txt

  # Importar múltiples archivos
  trazar archivo importar -t asistencias -r clase-001.txt clase-002.txt

  # Importar directorio completo (sin preguntar)
  trazar archivo importar -t asistencias -r ./asistencias/ -s

  # Importar con glob (el shell expande los *.txt)
  trazar archivo importar -t asistencias -r *.txt -s",
        after_help = "ESTRUCTURA DE DESTINO:
  Con asignatura:  datos/cursos/<id-curso>/archivo/asistencias/<id>-<nombre-kebab>/clase-<NNN>.txt
  Sin asignatura:  datos/cursos/<id-curso>/archivo/asistencias/clase-<NNN>.txt

NOTA: Las opciones -t, -r y -s pueden usarse en cualquier orden."
    )]
    Importar {
        /// Tipo de dataset (asistencias, quizzes, asignaciones, pagos)
        #[arg(short = 't', long = "tipo", value_name = "TIPO", required = true)]
        tipo: TipoDatasetCli,
        /// Ruta(s) del archivo(s) a importar (acepta múltiples archivos, globs, o directorios)
        #[arg(short = 'r', long = "ruta", value_name = "RUTA", required = true, num_args = 1..)]
        archivos: Vec<String>,
        /// Auto-afirmación: importa solo los archivos válidos sin preguntar
        #[arg(short = 's', long = "si")]
        si: bool,
    },
    
    /// Exportar datos consolidados
    #[command(
        help_template = HELP_SUBCMD,
        override_usage = "trazar archivo exportar <TIPO>"
    )]
    Exportar {
        /// Tipo de dataset (asistencias, quizzes, asignaciones, pagos)
        #[arg(value_name = "TIPO")]
        tipo: String,
    },
    
    /// Listar archivos en datos/archivo/
    #[command(
        help_template = HELP_SUBCMD,
        override_usage = "trazar archivo mostrar [TIPO]"
    )]
    Mostrar {
        /// Tipo de dataset (opcional, si se omite muestra todos)
        #[arg(value_name = "TIPO")]
        tipo: Option<String>,
    },
    
    /// Remover archivo de datos/archivo/
    #[command(
        help_template = HELP_SUBCMD,
        override_usage = "trazar archivo remover <ARCHIVO>"
    )]
    Remover {
        /// Ruta o nombre del archivo a remover
        #[arg(value_name = "ARCHIVO")]
        archivo: String,
    },
}

fn main() {
    let cli = Cli::parse();
    let ruta_base = obtener_ruta_base();
    
    match cli.command {
        Commands::Inspector { action } => {
            match action {
                InspectorAction::Init => {
                    match inspector::init(&ruta_base) {
                        Ok(_) => println!("✓ Estructura base creada/verificada"),
                        Err(e) => eprintln!("✗ Error: {}", e),
                    }
                }
                InspectorAction::Verificar => {
                    match inspector::verificar(&ruta_base) {
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
                InspectorAction::Purgar => {
                    let mut rl = match rustyline::DefaultEditor::new() {
                        Ok(editor) => editor,
                        Err(e) => {
                            eprintln!("✗ Error al inicializar editor: {}", e);
                            return;
                        }
                    };
                    
                    match rl.readline("¿Confirma purgar todos los datos? (purgar-todo/N): ") {
                        Ok(confirmacion) => {
                            if confirmacion.trim() == "purgar-todo" {
                                match inspector::purgar(&ruta_base) {
                                    Ok(_) => println!("✓ Datos eliminados"),
                                    Err(e) => eprintln!("✗ Error: {}", e),
                                }
                            } else {
                                println!("Operación cancelada. No se borró nada.");
                            }
                        }
                        Err(_) => {
                            println!("\nOperación cancelada. No se borró nada.");
                        }
                    }
                }
				InspectorAction::Validar { tipo } => {
					match inspector::validar(&ruta_base, tipo.as_deref()) {
						Ok(_) => {},
						Err(e) => eprintln!("✗ Error: {}", e),
					}
				}
				InspectorAction::Consolidar { curso, tipo } => {
					match inspector::consolidar(&ruta_base, &curso, tipo.as_deref()) {
						Ok(_) => {},
						Err(e) => eprintln!("✗ Error: {}", e),
					}
				}
            }
        }
        
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

			Commands::Archivo { action } => {
				match action {
					ArchivoAction::Importar { tipo, archivos, si } => {
						let tipo_str = tipo.to_string();
						match archivo::importar(&ruta_base, &tipo_str, &archivos, si) {
						Ok(_) => {},
						Err(e) => eprintln!("✗ Error: {}", e),
					}
				}
				ArchivoAction::Exportar { tipo } => {
					match archivo::exportar(&ruta_base, &tipo) {
						Ok(_) => {},
						Err(e) => eprintln!("✗ Error: {}", e),
					}
				}
				ArchivoAction::Mostrar { tipo } => {
					match archivo::mostrar(&ruta_base, tipo.as_deref()) {
						Ok(_) => {},
						Err(e) => eprintln!("✗ Error: {}", e),
					}
				}
				ArchivoAction::Remover { archivo } => {
					match archivo::remover(&ruta_base, &archivo) {
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
                    eprintln!("   trazar inspector <TAB>\n");
                }
                Shell::Bash => {
                    eprintln!("1. Crear el directorio si no existe:");
                    eprintln!("   mkdir -p ~/.bash_completion.d/\n");
                    eprintln!("2. Ejecutar este comando:");
                    eprintln!("   trazar completions bash ~/.bash_completion.d/trazar\n");
                    eprintln!("3. Activar:");
                    eprintln!("   source ~/.bash_completion.d/trazar\n");
                    eprintln!("4. Probar:");
                    eprintln!("   trazar inspector <TAB>\n");
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
                    eprintln!("   trazar inspector <TAB>\n");
                }
                Shell::PowerShell => {
                    eprintln!("1. Ejecutar este comando:");
                    eprintln!("   trazar completions powershell $PROFILE\n");
                    eprintln!("2. Reiniciar PowerShell\n");
                    eprintln!("3. Probar:");
                    eprintln!("   trazar inspector <TAB>\n");
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