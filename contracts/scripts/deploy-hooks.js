const hre = require("hardhat");
const fs = require("fs");
const path = require("path");

async function main() {
    const [deployer] = await hre.ethers.getSigners();

    console.log("========================================");
    console.log("ZK-ORIGIN HOOKS DEPLOYMENT");
    console.log("========================================");
    console.log("Deployer:", deployer.address);

    // Load previous deployment
    const deploymentPath = path.join(
        __dirname,
        "../deployments/localhost.json"
    );

    const deployment = JSON.parse(
        fs.readFileSync(deploymentPath, "utf8")
    );

    console.log("\nLoaded existing deployment");
    console.log(
        "Groth16Verifier:",
        deployment.contracts.Groth16Verifier
    );

    // -------------------------------------------------
    // Deploy MockPoolManager
    // -------------------------------------------------

    let poolManagerAddress = process.env.POOL_MANAGER_ADDRESS;

    if (!poolManagerAddress) {
        console.log("\nDeploying MockPoolManager for local testing...");

        const MockPoolManager = await hre.ethers.getContractFactory(
            "MockPoolManager"
        );

        const mockPoolManager = await MockPoolManager.deploy();

        await mockPoolManager.deployed();

        poolManagerAddress = mockPoolManager.address;

        console.log("MockPoolManager:", poolManagerAddress);
    }

    // -------------------------------------------------
    // Deploy Donation Verifier
    // -------------------------------------------------

    console.log("\nStep 1: Deploying DonationVerifier...");

    const MockDonationVerifier = await hre.ethers.getContractFactory(
        "MockDonationVerifier"
    );

    const donationVerifier = await MockDonationVerifier.deploy();

    await donationVerifier.deployed();

    const donationVerifierAddress = donationVerifier.address;

    console.log("DonationVerifier:", donationVerifierAddress);

    // -------------------------------------------------
    // Deploy Permission Verifier
    // -------------------------------------------------

    console.log("\nStep 2: Deploying PermissionVerifier...");

    const MockPermissionVerifier = await hre.ethers.getContractFactory(
        "MockPermissionVerifier"
    );

    const permissionVerifier = await MockPermissionVerifier.deploy();

    await permissionVerifier.deployed();

    const permissionVerifierAddress = permissionVerifier.address;

    console.log("PermissionVerifier:", permissionVerifierAddress);

    // -------------------------------------------------
    // Deploy ZKOriginDonationHook
    // Constructor expects:
    // (poolManager, donationVerifier)
    // -------------------------------------------------

    console.log("\nStep 3: Deploying ZKOriginDonationHook...");

    const DonationHook = await hre.ethers.getContractFactory(
        "ZKOriginDonationHook"
    );

    const donationHook = await DonationHook.deploy(
        poolManagerAddress,
        donationVerifierAddress
    );

    await donationHook.deployed();

    const donationHookAddress = donationHook.address;

    console.log("ZKOriginDonationHook:", donationHookAddress);

    // -------------------------------------------------
    // Deploy ZKOriginPermissionHook
    // Constructor expects:
    // (permissionVerifier)
    // -------------------------------------------------

    console.log("\nStep 4: Deploying ZKOriginPermissionHook...");

    const PermissionHook = await hre.ethers.getContractFactory(
        "ZKOriginPermissionHook"
    );

    const permissionHook = await PermissionHook.deploy(
        permissionVerifierAddress
    );

    await permissionHook.deployed();

    const permissionHookAddress = permissionHook.address;

    console.log("ZKOriginPermissionHook:", permissionHookAddress);

    // -------------------------------------------------
    // Save deployment
    // -------------------------------------------------

    const hookDeployment = {
        network: hre.network.name,
        timestamp: new Date().toISOString(),
        deployer: deployer.address,
        contracts: {
            PoolManager: poolManagerAddress,
            DonationVerifier: donationVerifierAddress,
            PermissionVerifier: permissionVerifierAddress,
            ZKOriginDonationHook: donationHookAddress,
            ZKOriginPermissionHook: permissionHookAddress
        }
    };

    const hookDeploymentPath = path.join(
        __dirname,
        "../deployments/localhost-hooks.json"
    );

    fs.writeFileSync(
        hookDeploymentPath,
        JSON.stringify(hookDeployment, null, 2)
    );

    console.log("\nHook deployment saved to:");
    console.log(hookDeploymentPath);

    console.log("\n========================================");
    console.log("HOOKS DEPLOYMENT COMPLETE");
    console.log("========================================");

    console.log("PoolManager            :", poolManagerAddress);
    console.log("DonationVerifier       :", donationVerifierAddress);
    console.log("PermissionVerifier     :", permissionVerifierAddress);
    console.log("ZKOriginDonationHook   :", donationHookAddress);
    console.log("ZKOriginPermissionHook :", permissionHookAddress);
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });