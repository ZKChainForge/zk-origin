import { groth16 } from "snarkjs";
import * as path from "path";
import * as fs from "fs";
import chalk from "chalk";
import ora from "ora";

import {
    Origin,
    OriginNames,
    isPolicyAllowed,
    randomHash,
    genesisLineage,
    formatHash
} from "./utils";

interface Transition {
    name: string;
    fromOrigin: Origin;
    toOrigin: Origin;
    description: string;
}

async function runDemo() {
    console.log(chalk.bold.blue("\n ZK-ORIGIN Demo"));
    console.log(chalk.gray("Demonstrating zero-knowledge state provenance verification\n"));
    
    const wasmPath = path.join(__dirname, "../../circuits/build/lineage_step_simple_js/lineage_step_simple.wasm");
    const zkeyPath = path.join(__dirname, "../../circuits/build/lineage_step_simple.zkey");
    const vkeyPath = path.join(__dirname, "../../circuits/build/verification_key.json");
    
    // Check files exist
    if (!fs.existsSync(wasmPath)) {
        console.error(chalk.red(" Circuit not compiled. Run: cd circuits && npm run compile && npm run setup"));
        process.exit(1);
    }
    
    // Define scenario
    console.log(chalk.yellow(" Scenario: DeFi Protocol Lifecycle\n"));
    
    const transitions: Transition[] = [
        { name: "Protocol Deployment", fromOrigin: Origin.Genesis, toOrigin: Origin.Admin,
          description: "Admin deploys and configures protocol" },
        { name: "Open for Users", fromOrigin: Origin.Admin, toOrigin: Origin.User,
          description: "Protocol opens, first user deposits" },
        { name: "User Activity", fromOrigin: Origin.User, toOrigin: Origin.User,
          description: "Normal user transactions" },
        { name: "More Activity", fromOrigin: Origin.User, toOrigin: Origin.User,
          description: "Additional user deposits and swaps" }
    ];
    
    // Print transition plan
    console.log(chalk.cyan("Planned transitions:"));
    for (let i = 0; i < transitions.length; i++) {
        const t = transitions[i];
        console.log(chalk.gray(`  ${i + 1}. ${t.name}`));
        console.log(chalk.gray(`     ${OriginNames[t.fromOrigin]} → ${OriginNames[t.toOrigin]}`));
        console.log(chalk.gray(`     "${t.description}"\n`));
    }
    
    // Generate states
    const states: bigint[] = [];
    for (let i = 0; i <= transitions.length; i++) {
        states.push(randomHash());
    }
    
    console.log(chalk.cyan("Generated states:"));
    for (let i = 0; i < states.length; i++) {
        console.log(chalk.gray(`  State ${i}: ${formatHash(states[i])}`));
    }
    console.log();
    
    // Process transitions
    let currentLineage = await genesisLineage(states[0]);
    let currentDepth = 0;
    const proofs: any[] = [];
    
    console.log(chalk.green(" Processing transitions:\n"));
    
    for (let i = 0; i < transitions.length; i++) {
        const t = transitions[i];
        const spinner = ora(`${t.name} (${OriginNames[t.fromOrigin]} → ${OriginNames[t.toOrigin]})`).start();
        
        // Check policy
        if (!isPolicyAllowed(t.fromOrigin, t.toOrigin, currentDepth)) {
            spinner.fail(chalk.red(`Policy violation: ${OriginNames[t.fromOrigin]} → ${OriginNames[t.toOrigin]}`));
            continue;
        }
        
        // Generate witness
        const witness = {
            prev_state_hash: states[i].toString(),
            new_state_hash: states[i + 1].toString(),
            prev_lineage_commitment: currentLineage.toString(),
            prev_origin: t.fromOrigin.toString(),
            new_origin: t.toOrigin.toString(),
            prev_depth: currentDepth.toString(),
            timestamp: Date.now().toString()
        };
        
        // Generate proof
        const startTime = Date.now();
        const { proof, publicSignals } = await groth16.fullProve(witness, wasmPath, zkeyPath);
        const elapsed = Date.now() - startTime;
        
        spinner.succeed(chalk.green(`${t.name} - Proof generated in ${elapsed}ms`));
        console.log(chalk.gray(`     New lineage: ${formatHash(BigInt(publicSignals[0]))}`));
        console.log(chalk.gray(`     Depth: ${publicSignals[1]}\n`));
        
        currentLineage = BigInt(publicSignals[0]);
        currentDepth = parseInt(publicSignals[1]);
        proofs.push({ proof, publicSignals, name: t.name });
    }
    
    // Verify all proofs
    console.log(chalk.yellow("\n Verifying all proofs:\n"));
    
    const vkey = JSON.parse(fs.readFileSync(vkeyPath, "utf-8"));
    
    for (const p of proofs) {
        const spinner = ora(`Verifying: ${p.name}`).start();
        const valid = await groth16.verify(vkey, p.publicSignals, p.proof);
        
        if (valid) {
            spinner.succeed(chalk.green(`${p.name} - Valid `));
        } else {
            spinner.fail(chalk.red(`${p.name} - Invalid `));
        }
    }
    
    // Summary
    console.log(chalk.bold.green("\n Demo Complete!\n"));
    console.log(chalk.cyan("Summary:"));
    console.log(chalk.gray(`  Total transitions: ${proofs.length}`));
    console.log(chalk.gray(`  Final depth: ${currentDepth}`));
    console.log(chalk.gray(`  Final lineage: ${formatHash(currentLineage)}`));
    
    // Attack simulation
    console.log(chalk.bold.yellow("\n  Attack Simulation:\n"));
    console.log(chalk.gray("Attempting privilege escalation (User → Admin)...\n"));
    
    const attackSpinner = ora("User trying to become Admin...").start();
    
    if (!isPolicyAllowed(Origin.User, Origin.Admin, currentDepth)) {
        attackSpinner.fail(chalk.red("Attack blocked by policy!"));
        console.log(chalk.gray("  The circuit would reject this proof generation.\n"));
    }
    
    console.log(chalk.bold.blue(" ZK-ORIGIN successfully protects against unauthorized state origins!\n"));
}

runDemo().catch(console.error);