use clap::{Parser, Subcommand, CommandFactory};
use clap_complete::{generate, Shell};
use std::path::PathBuf;
use rustyline::DefaultEditor;

mod inspector;
mod curso;

#[derive(Parser)]
#[command(name = "trazar", about = "Trazabilidad Académica y Reporte", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Gestión de estructura de datos
    Inspector {
        /// Crear estructura base
        #[arg(short = 'e', long = "estructurar")]
        estructurar: bool,
        
        /// Verificar integridad
        #[arg(short = 'r', long = "revisar")]
        revisar: bool,
        
        /// Borrar datos de usuario
        #[arg(short = 'b', long = "borrar")]
        limpiar: bool,
    },
    
	/// Gestión de cursos
	Curso {
		/// Agregar curso
		#[arg(short = 'a', long = "agregar")]
		agregar: bool,
		
		/// Ver cursos -opcionalmente por ID o nombre-
		#[arg(short = 'v', long = "ver", num_args = 0..=1, default_missing_value = "")]
		ver: Option<String>,
		
		/// Editar curso -opcionalmente por ID o nombre-
		#[arg(short = 'e', long = "editar", num_args = 0..=1, default_missing_value = "")]
		editar: Option<String>,
		
		/// Borrar cursos
		#[arg(short = 'b', long = "borrar", num_args = 1..)]
		borrar: Vec<String>,
	},
    
    /// Generar scripts de autocompletado
    Completions {
        /// Shell para el que generar el script
        shell: Shell,
        /// Ruta donde guardar el script
        ruta: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    let ruta_base = obtener_ruta_base();

    match cli.command {
        Commands::Inspector { estructurar, revisar, limpiar } => {
            if estructurar {
                match inspector::estructurar_base(&ruta_base) {
                    Ok(_) => println!("✓ Estructura base creada/verificada"),
                    Err(e) => eprintln!("✗ Error: {}", e),
                }
            } else if revisar {
                match inspector::revisar_integridad(&ruta_base) {
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
            } else if limpiar {
				let mut rl = match DefaultEditor::new() {
					Ok(editor) => editor,
					Err(e) => {
						eprintln!("✗ Error al inicializar editor: {}", e);
						return;
					}
				};
				
				match rl.readline("¿Está seguro de borrar todos los datos? (borrar-todo/N): ") {
					Ok(confirmacion) => {
						if confirmacion.trim() == "borrar-todo" {
							match inspector::limpiar(&ruta_base) {
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
			} else {
                eprintln!("Debe especificar una acción: -e, -r, -b");
                eprintln!("Use 'trazar inspector --help' para más información");
            }
        }
        
		Commands::Curso { agregar, ver, editar, borrar } => {
			if agregar {
				match curso::agregar(&ruta_base) {
					Ok(_) => {},
					Err(e) => eprintln!("✗ Error: {}", e),
				}
			} else if let Some(ref argumento) = ver {
				if argumento.is_empty() {
					match curso::ver(&ruta_base, None) {
						Ok(_) => {},
						Err(e) => eprintln!("✗ Error: {}", e),
					}
				} else {
					match curso::ver(&ruta_base, Some(argumento.as_str())) {
						Ok(_) => {},
						Err(e) => eprintln!("✗ Error: {}", e),
					}
				}
			} else if let Some(ref argumento) = editar {
				if argumento.is_empty() {
					match curso::editar(&ruta_base, None) {
						Ok(_) => {},
						Err(e) => eprintln!("✗ Error: {}", e),
					}
				} else {
					match curso::editar(&ruta_base, Some(argumento.as_str())) {
						Ok(_) => {},
						Err(e) => eprintln!("✗ Error: {}", e),
					}
				}
			} else if !borrar.is_empty() {
				match curso::borrar(&ruta_base, &borrar) {
					Ok(_) => {},
					Err(e) => eprintln!("✗ Error: {}", e),
				}
			} else {
				eprintln!("Debe especificar una acción: -a, -v, -e, -b");
				eprintln!("Use 'trazar curso --help' para más información");
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