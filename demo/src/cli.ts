import { Command } from "commander";
import { groth16 } from "snarkjs";
import * as path from "path";
import * as fs from "fs";
import chalk from "chalk";

import {
    Origin,
    OriginNames,
    isPolicyAllowed,
    randomHash,
    genesisLineage,
    formatHash
} from "./utils";

const program = new Command();

program
    .name("zk-origin")
    .description("ZK-ORIGIN CLI - Zero-Knowledge State Provenance")
    .version("1.0.0");

program
    .command("prove")
    .description("Generate a lineage proof")
    .option("-f, --from <origin>", "From origin class (Genesis, User, Admin, Bridge)", "Genesis")
    .option("-t, --to <origin>", "To origin class", "User")
    .option("-d, --depth <number>", "Current depth", "0")
    .action(async (options) => {
        console.log(chalk.blue("\n ZK-ORIGIN Proof Generation\n"));
        
        const wasmPath = path.join(__dirname, "../../circuits/build/lineage_step_simple_js/lineage_step_simple.wasm");
        const zkeyPath = path.join(__dirname, "../../circuits/build/lineage_step_simple.zkey");
        
        if (!fs.existsSync(wasmPath)) {
            console.error(chalk.red(" Circuit not compiled. Run setup first."));
            process.exit(1);
        }
        
        const fromOrigin = Origin[options.from as keyof typeof Origin];
        const toOrigin = Origin[options.to as keyof typeof Origin];
        const depth = parseInt(options.depth);
        
        console.log(`From: ${OriginNames[fromOrigin]}`);
        console.log(`To: ${OriginNames[toOrigin]}`);
        console.log(`Depth: ${depth}\n`);
        
        if (!isPolicyAllowed(fromOrigin, toOrigin, depth)) {
            console.error(chalk.red(`Policy violation: ${OriginNames[fromOrigin]} → ${OriginNames[toOrigin]}`));
            process.exit(1);
        }
        
        const prevState = randomHash();
        const newState = randomHash();
        const lineage = await genesisLineage(prevState);
        
        const witness = {
            prev_state_hash: prevState.toString(),
            new_state_hash: newState.toString(),
            prev_lineage_commitment: lineage.toString(),
            prev_origin: fromOrigin.toString(),
            new_origin: toOrigin.toString(),
            prev_depth: depth.toString(),
            timestamp: Date.now().toString()
        };
        
        console.log("Generating proof...");
        const start = Date.now();
        const { proof, publicSignals } = await groth16.fullProve(witness, wasmPath, zkeyPath);
        const elapsed = Date.now() - start;
        
        console.log(chalk.green(`\n Proof generated in ${elapsed}ms`));
        console.log(`New lineage: ${formatHash(BigInt(publicSignals[0]))}`);
        console.log(`New depth: ${publicSignals[1]}`);
        
        // Save proof
        const outputPath = path.join(__dirname, "../../circuits/build/cli_proof.json");
        fs.writeFileSync(outputPath, JSON.stringify({ proof, publicSignals }, null, 2));
        console.log(`\nProof saved to: ${outputPath}`);
    });

program
    .command("verify")
    .description("Verify a lineage proof")
    .option("-p, --proof <path>", "Path to proof file", "../../circuits/build/cli_proof.json")
    .action(async (options) => {
        console.log(chalk.blue("\n🔍 ZK-ORIGIN Proof Verification\n"));
        
        const vkeyPath = path.join(__dirname, "../../circuits/build/verification_key.json");
        const proofPath = path.resolve(__dirname, options.proof);
        
        if (!fs.existsSync(vkeyPath)) {
            console.error(chalk.red(" Verification key not found. Run setup first."));
            process.exit(1);
        }
        
        if (!fs.existsSync(proofPath)) {
            console.error(chalk.red(` Proof file not found: ${proofPath}`));
            process.exit(1);
        }
        
        const vkey = JSON.parse(fs.readFileSync(vkeyPath, "utf-8"));
        const { proof, publicSignals } = JSON.parse(fs.readFileSync(proofPath, "utf-8"));
        
        console.log("Verifying proof...");
        const start = Date.now();
        const valid = await groth16.verify(vkey, publicSignals, proof);
        const elapsed = Date.now() - start;
        
        if (valid) {
            console.log(chalk.green(`\n Proof is valid! (${elapsed}ms)`));
            console.log(`Lineage commitment: ${formatHash(BigInt(publicSignals[0]))}`);
            console.log(`Depth: ${publicSignals[1]}`);
        } else {
            console.log(chalk.red(`\n Proof is invalid!`));
            process.exit(1);
        }
    });

program
    .command("policy")
    .description("Show the current origin policy")
    .action(() => {
        console.log(chalk.blue("\n📋 ZK-ORIGIN Policy Matrix\n"));
        
        const classes = ["Genesis", "User", "Admin", "Bridge"];
        
        // Header
        console.log("         " + classes.map(c => c.padEnd(8)).join(" "));
        console.log("        " + "-".repeat(40));
        
        for (let from = 0; from < 4; from++) {
            let row = classes[from].padEnd(8) + " |";
            for (let to = 0; to < 4; to++) {
                const allowed = isPolicyAllowed(from, to, from === 0 ? 0 : 1);
                row += (allowed ? chalk.green("  ✓  ") : chalk.red("  ✗  ")) + "  ";
            }
            console.log(row);
        }
        
        console.log("\n" + chalk.gray(" = Allowed,  = Blocked"));
        console.log(chalk.gray("\nNote: No origin can return to Genesis after depth > 0"));
    });

program
    .command("demo")
    .description("Run the full demo")
    .action(async () => {
        const demoPath = path.join(__dirname, "demo.ts");
        require(demoPath);
    });

program.parse();