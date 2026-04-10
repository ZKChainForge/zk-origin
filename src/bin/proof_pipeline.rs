use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║        ZK-ORIGIN Full Proof Generation Pipeline        ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    // Step 1: Generate witness
    println!(" Step 1: Generating witness...");
    generate_witness()?;

    // Step 2: Build circuit
    println!("\n Step 2: Compiling circuit...");
    compile_circuit()?;

    // Step 3: Generate proof
    println!("\n Step 3: Generating proof...");
    generate_proof()?;

    // Step 4: Verify proof
    println!("\n  Step 4: Verifying proof...");
    verify_proof()?;

    println!("\n╔════════════════════════════════════════════════════════╗");
    println!("║               PROOF PIPELINE COMPLETE                 ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    println!(" Generated files:");
    println!("   Witness: circuits/build/witness.wtns");
    println!("   Proof: circuits/build/proof.json");
    println!("   Public signals: circuits/build/public.json");

    Ok(())
}

fn generate_witness() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "generate_witness", "--release"])
        .output()?;

    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        return Err("Witness generation failed".into());
    }

    print!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

fn compile_circuit() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("circom")
        .args(&[
            "circuits/src/main/main.circom",
            "--r1cs",
            "--wasm",
            "--sym",
            "-o",
            "circuits/build/",
        ])
        .output()?;

    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        return Err("Circuit compilation failed".into());
    }



    Ok(())
}

fn generate_proof() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("bash")
        .arg("-c")
        .arg(
            "cd circuits/build && \
             snarkjs wtns calculate main_js/main.wasm ../test/inputs/first_transition_input.json witness.wtns && \
             snarkjs groth16 prove main_0000.zkey witness.wtns proof.json public.json",
        )
        .output()?;

    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        return Err("Proof generation failed".into());
    }


    Ok(())
}

fn verify_proof() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("bash")
        .arg("-c")
        .arg(
            "cd circuits/build && \
             snarkjs groth16 verify verification_key.json public.json proof.json",
        )
        .output()?;

    if output.status.success() {
        println!(" Proof verified successfully!");
    } else {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        return Err("Proof verification failed".into());
    }

    Ok(())
}