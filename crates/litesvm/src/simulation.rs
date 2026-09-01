use {
    crate::{
        new_log_collector, simulation_outcome,
        types::{ExecutionResult, FailedTransactionMetadata, SimulatedTransactionInfo},
        LiteSVM,
    },
    solana_transaction::sanitized::SanitizedTransaction,
};

/// One transaction's simulation, cut loose from the instance that prepared it
pub struct PreparedSimulation {
    prepared: Result<(LiteSVM, SanitizedTransaction), ExecutionResult>,
    log_bytes_limit: Option<usize>,
}

impl PreparedSimulation {
    pub(crate) fn new(
        prepared: Result<(LiteSVM, SanitizedTransaction), ExecutionResult>,
        log_bytes_limit: Option<usize>,
    ) -> Self {
        Self {
            prepared,
            log_bytes_limit,
        }
    }

    /// Executes against the copied accounts, touching nothing the instance holds
    pub fn run(self) -> Result<SimulatedTransactionInfo, FailedTransactionMetadata> {
        let log_collector = new_log_collector(self.log_bytes_limit);
        let result = match self.prepared {
            Ok((view, sanitized)) => {
                view.execute_sanitized_transaction_readonly(&sanitized, log_collector.clone())
            }
            Err(rejected) => rejected,
        };
        simulation_outcome(result, log_collector)
    }
}
