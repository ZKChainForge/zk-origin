const hre = require("hardhat");
const fs = require("fs");
const path = require("path");
const { getContractInstance, getDeploymentMetadata } = require("./helpers/deployment");

async function submitProof() {
    console.log("╔════════════════════════════════════════════════════════╗");
    console.log("║       ZK-ORIGIN Proof Submission to Contract           ║");
    console.log("╚════════════════════════════════════════════════════════╝\n");

    const [signer] = await hre.ethers.getSigners();
    const balance = await signer.getBalance();

    console.log(" Submission Details:");
    console.log("   Signer:", signer.address);
    console.log("   Balance:", hre.ethers.utils.formatEther(balance), "ETH\n");

    try {
        // Load deployment
        const metadata = getDeploymentMetadata();
        console.log(" Deployment Info:");
        console.log("   Network:", metadata.network);
        console.log("   Chain ID:", metadata.chainId);
        console.log("   Deployer:", metadata.deployer);
        console.log("   Timestamp:", metadata.timestamp, "\n");

        // Get contract instance
        const lineageVerifier = await getContractInstance(hre, "LineageVerifier");
        
        console.log(" Contracts Loaded:");
        console.log("   LineageVerifier:", lineageVerifier.address, "\n");

        // Example proof data (replace with actual proof)
        const proofData = {
            a: [
                "0x0000000000000000000000000000000000000000000000000000000000000001",
                "0x0000000000000000000000000000000000000000000000000000000000000002"
            ],
            b: [
                [
                    "0x0000000000000000000000000000000000000000000000000000000000000003",
                    "0x0000000000000000000000000000000000000000000000000000000000000004"
                ],
                [
                    "0x0000000000000000000000000000000000000000000000000000000000000005",
                    "0x0000000000000000000000000000000000000000000000000000000000000006"
                ]
            ],
            c: [
                "0x0000000000000000000000000000000000000000000000000000000000000007",
                "0x0000000000000000000000000000000000000000000000000000000000000008"
            ]
        };

        const inputs = ["0x0000000000000000000000000000000000000000000000000000000000000001"];

        console.log(" Submitting proof...");
        console.log("   Proof A:", proofData.a);
        console.log("   Proof B:", proofData.b);
        console.log("   Proof C:", proofData.c);
        console.log("   Inputs:", inputs, "\n");

       
        // const tx = await lineageVerifier.verify(proofData.a, proofData.b, proofData.c, inputs);
        // const receipt = await tx.wait();
        
 

    } catch (error) {
        console.error("\n Error:", error.message);
        if (error.stack) {
            console.error("\nStack trace:");
            console.error(error.stack);
        }
        process.exit(1);
    }
}

submitProof()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("\n Fatal error:", error);
        process.exit(1);
    });