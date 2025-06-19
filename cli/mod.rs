pub mod dashboard;
pub mod deployment;
pub mod smart_contracts;
pub mod monitoring;
pub mod network;
pub mod security;
pub mod testing;

pub use dashboard::show_production_dashboard;
pub use deployment::handle_deployment;
pub use smart_contracts::handle_smart_contracts;
pub use monitoring::show_monitoring_system;
pub use network::show_network_status;
pub use security::show_security_status;
pub use testing::run_integration_tests;

pub fn show_help() {
    println!("🔧 BCAI CLI Commands:");
    println!("   dashboard  - Show production dashboard");
    println!("   deploy     - Deployment management");
    println!("   contract   - Smart contract operations");
    println!("   monitor    - System monitoring");
    println!("   network    - Network management");
    println!("   security   - Security operations");
    println!("   test       - Run integration tests");
} 