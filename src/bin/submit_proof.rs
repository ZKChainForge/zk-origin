use std::fs;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║       ZK-ORIGIN Proof Submission Helper                ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    // Read proof files
    println!(" Reading proof files...\n");

    let proof_json =
        fs::read_to_string("circuits/build/proof.json")?;
    let proof: serde_json::Value = serde_json::from_str(&proof_json)?;

    let public_json =
        fs::read_to_string("circuits/build/public.json")?;
    let public: Vec<String> = serde_json::from_str(&public_json)?;



    println!(" Public signals loaded");
    println!("   Count: {}\n", public.len());

    // Format for Solidity contract
    println!(" Formatting for Solidity contract...\n");

    let p_a = vec![
        proof["pi_a"][0].as_str().unwrap_or("0"),
        proof["pi_a"][1].as_str().unwrap_or("0"),
    ];

    let p_b = vec![
        vec![
            proof["pi_b"][0][1].as_str().unwrap_or("0"),
            proof["pi_b"][0][0].as_str().unwrap_or("0"),
        ],
        vec![
            proof["pi_b"][1][1].as_str().unwrap_or("0"),
            proof["pi_b"][1][0].as_str().unwrap_or("0"),
        ],
    ];

    let p_c = vec![
        proof["pi_c"][0].as_str().unwrap_or("0"),
        proof["pi_c"][1].as_str().unwrap_or("0"),
    ];

    // Create submission script content
    let script_content = format!(
        r#"
const proofData = {{
  pA: ["{}", "{}"],
  pB: [["{}", "{}"], ["{}", "{}"]],
  pC: ["{}", "{}"],
  publicSignals: [{}]
}};

console.log("Proof Data for Contract:");
console.log(JSON.stringify(proofData, null, 2));

// Copy to contracts/scripts/submit-proof.js and run:
// npx hardhat run scripts/submit-proof.js --network localhost
"#,
        p_a[0], p_a[1], 
        p_b[0][0], p_b[0][1], 
        p_b[1][0], p_b[1][1], 
        p_c[0], p_c[1],
        public.iter().map(|s| format!(r#""{}""#, s)).collect::<Vec<_>>().join(", ")
    );

    // Save to file
    fs::write("proof_data.js", script_content)?;



    // Create JSON for hardhat script
    let submission = json!({
        "pA": p_a,
        "pB": p_b,
        "pC": p_c,
        "publicSignals": public
    });

    fs::write(
        "proof_submission.json",
        serde_json::to_string_pretty(&submission)?,
    )?;



    Ok(())
}