// MÓDULO INSPECTOR

// Declarar submódulos (herramientas)
mod estructuradora_base;
mod revisora_integridad;
mod limpiadora;

// Re-exportar funciones públicas para que main.rs pueda usarlas
pub use estructuradora_base::ejecutar as estructurar_base;
pub use revisora_integridad::ejecutar as revisar_integridad;
pub use limpiadora::ejecutar as limpiar;