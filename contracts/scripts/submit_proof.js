/**
 * Submit Proof: Take Groth16 proof and submit to LineageVerifier
 * 
 * Usage:
 *   node submit_proof.js --proof proof.json --signals public.json
 */

const { ethers } = require("hardhat");
const fs = require("fs");
const path = require("path");

async function main() {
    // Parse arguments
    const args = process.argv.slice(2);
    let proofPath = "circuits/build/proof.json";
    let signalsPath = "circuits/build/public.json";
    
    for (let i = 0; i < args.length; i++) {
        if (args[i] === "--proof") proofPath = args[i + 1];
        if (args[i] === "--signals") signalsPath = args[i + 1];
    }
    
    console.log(`\n${'═'.repeat(60)}`);
    console.log(" ZK-ORIGIN PROOF SUBMISSION");
    console.log(`${'═'.repeat(60)}\n`);
    
    // Load proof
    console.log(" Loading proof files...");
    if (!fs.existsSync(proofPath)) {
        throw new Error(`Proof file not found: ${proofPath}`);
    }
    if (!fs.existsSync(signalsPath)) {
        throw new Error(`Signals file not found: ${signalsPath}`);
    }
    
    const proof = JSON.parse(fs.readFileSync(proofPath, "utf8"));
    const signals = JSON.parse(fs.readFileSync(signalsPath, "utf8"));
    
    console.log(` Proof loaded`);
    console.log(`Signals loaded (${signals.length} inputs)`);
    
    // Get contract
    console.log("\n Connecting to LineageVerifier...");
    const lineageVerifier = await ethers.getContractAt(
        "LineageVerifier",
        process.env.LINEAGE_VERIFIER || "0xDc64a140Aa3E981100a9becA4E685f962f0cF6C9"
    );
    console.log(` Connected to: ${lineageVerifier.address}`);
    
    // Format proof
    console.log("\n Formatting proof...");
    const formattedProof = formatProof(proof, signals);
    console.log(` Proof formatted`);
    console.log(`   pA: [${formattedProof.pA.map(n => n.toString().slice(0, 10)).join(", ")}...]`);
    console.log(`   pB: [[...], [...]]`);
    console.log(`   pC: [${formattedProof.pC.map(n => n.toString().slice(0, 10)).join(", ")}...]`);
    
    // Submit proof
    console.log("\n Submitting proof to contract...");
    const tx = await lineageVerifier.verifyLineage(
        formattedProof.pA,
        formattedProof.pB,
        formattedProof.pC,
        formattedProof.publicInputs
    );
    
    console.log(` Transaction hash: ${tx.hash}`);
    console.log(` Waiting for confirmation...`);
    
    const receipt = await tx.wait();
    
    
    if (receipt.status === 1) {
        console.log(" PROOF VERIFIED SUCCESSFULLY!");
    } else {
        console.log(" PROOF VERIFICATION FAILED");
    }
 
    
    console.log(`\n Transaction Details:`);
    console.log(`   Status: ${receipt.status === 1 ? "Success" : "Failed"}`);
    console.log(`   Gas used: ${receipt.gasUsed.toString()}`);
    console.log(`   Block: ${receipt.blockNumber}`);
    console.log(`   Events: ${receipt.logs.length}`);
    
    // Parse and display events
    if (receipt.logs.length > 0) {
        console.log(`\n Events:`);
        for (const log of receipt.logs) {
            try {
                const parsed = lineageVerifier.interface.parseLog(log);
                console.log(`   - ${parsed.name}:`);
                for (const param of parsed.args) {
                    console.log(`     ${param}`);
                }
            } catch (e) {
                // Event from other contract
            }
        }
    }
    
    // Save results
    const results = {
        proof: {
            file: proofPath,
            hash: proof,
        },
        signals: {
            file: signalsPath,
            count: signals.length,
            values: signals.slice(0, 3).map(s => s.toString().slice(0, 20)),
        },
        transaction: {
            hash: tx.hash,
            block: receipt.blockNumber,
            gasUsed: receipt.gasUsed.toString(),
            status: receipt.status === 1 ? "success" : "failed",
        },
        timestamp: new Date().toISOString(),
    };
    
    fs.writeFileSync("proof_submission_result.json", JSON.stringify(results, null, 2));
    console.log(`\n Results saved to proof_submission_result.json`);
}

/**
 * Format snarkjs proof to Solidity format
 */
function formatProof(proof, signals) {
    const pA = [
        BigInt(proof.pi_a[0]),
        BigInt(proof.pi_a[1]),
    ];
    
    const pB = [
        [BigInt(proof.pi_b[0][1]), BigInt(proof.pi_b[0][0])],
        [BigInt(proof.pi_b[1][1]), BigInt(proof.pi_b[1][0])],
    ];
    
    const pC = [
        BigInt(proof.pi_c[0]),
        BigInt(proof.pi_c[1]),
    ];
    
    const publicInputs = signals.map(s => BigInt(s));
    
    return {
        pA,
        pB,
        pC,
        publicInputs,
    };
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });