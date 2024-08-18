use rand::Rng;

use super::screen_constants::PAYMENT_ACCEPTANCE_RATE;

/// `ScreenPaymentSystem` simula el sistema de captura de pagos con una tasa de aceptación configurable.
pub struct ScreenPaymentSystem {
    acceptance_rate: f32,
}

impl Default for ScreenPaymentSystem {
    /// Crea una nueva instancia de `ScreenPaymentSystem` con una tasa de aceptación predeterminada del 95%.
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenPaymentSystem {
    /// Crea una nueva instancia de `ScreenPaymentSystem` con una tasa de aceptación del 95% por defecto.
    ///
    /// # Retorna
    ///
    /// Una nueva instancia de `ScreenPaymentSystem` con una tasa de aceptación del 95%.
    pub fn new() -> Self {
        ScreenPaymentSystem {
            acceptance_rate: PAYMENT_ACCEPTANCE_RATE,
        }
    }

    /// Establece la tasa de aceptación a 0%, utilizado únicamente para pruebas.
    ///
    /// # Nota
    ///
    /// Este método es privado y solo debe ser utilizado en entornos de prueba.
    fn _test_set_deny_all(&mut self) {
        self.acceptance_rate = 0.0;
    }

    /// Intenta realizar la captura del pago con base en la tasa de aceptación configurada.
    ///
    /// # Retorna
    ///
    /// `true` si la captura del pago fue exitosa según la tasa de aceptación actual,
    /// o `false` si fue rechazada.
    pub fn attempt_payment_capture(&self) -> bool {
        let mut rng = rand::thread_rng();
        let random_number: f32 = rng.gen_range(0.0..=1.0);
        random_number <= self.acceptance_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Prueba para verificar que `attempt_payment_capture` acepta al menos un pago en tres intentos con la tasa de aceptación por defecto.
    #[test]
    fn test_attempt_payment_capture_accept() {
        let payment_system = ScreenPaymentSystem::new();
        let mut one_accepted = false;
        for _ in 0..3 {
            let result = payment_system.attempt_payment_capture();
            if result {
                one_accepted = true;
                break;
            }
        }
        assert_eq!(one_accepted, true);
    }

    /// Prueba para verificar que `attempt_payment_capture` rechaza todos los pagos cuando la tasa de aceptación se establece en 0%.
    #[test]
    fn test_attempt_payment_capture_deny() {
        let mut payment_system = ScreenPaymentSystem::new();
        payment_system._test_set_deny_all();
        let result = payment_system.attempt_payment_capture();
        assert_eq!(result, false);
    }

    /// Prueba para verificar que `attempt_payment_capture` rechaza al menos un pago en 10,000 intentos con la tasa de aceptación por defecto.
    #[test]
    fn test_attempt_payment_capture_at_least_one_denys() {
        let payment_system = ScreenPaymentSystem::new();
        let mut one_denied = false;
        for _ in 0..10000 {
            let result = payment_system.attempt_payment_capture();
            if !result {
                one_denied = true;
                break;
            }
        }
        assert_eq!(one_denied, true);
    }
}
