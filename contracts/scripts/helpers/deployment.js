const fs = require("fs");
const path = require("path");

/**
 * Load deployment configuration from file
 * @returns {Object} Deployment object with contract addresses and metadata
 */
function loadDeployment() {
    const deploymentFile = path.join(__dirname, "../../deployment-complete.json");
    
    if (!fs.existsSync(deploymentFile)) {
        throw new Error(
            "Deployment file not found at: " + deploymentFile + 
            "\nPlease run: npm run deploy"
        );
    }

    try {
        return JSON.parse(fs.readFileSync(deploymentFile, "utf8"));
    } catch (error) {
        throw new Error("Failed to parse deployment file: " + error.message);
    }
}

/**
 * Get contract instance from deployed address
 * @param {Object} hre - Hardhat runtime environment
 * @param {string} contractName - Name of the contract
 * @returns {Promise<Object>} Contract instance attached to deployed address
 */
async function getContractInstance(hre, contractName) {
    const deployments = loadDeployment();
    const address = deployments[contractName];
    
    if (!address) {
        throw new Error(
            `Contract "${contractName}" not found in deployments.\n` +
            `Available contracts: ${Object.keys(deployments)
                .filter(k => !['network', 'chainId', 'deployer', 'timestamp', 'genesisStateHash', 'genesisLineageCommitment', 'policyRoot', 'transitionCount'].includes(k))
                .join(", ")}`
        );
    }

    try {
        const factory = await hre.ethers.getContractFactory(contractName);
        return factory.attach(address);
    } catch (error) {
        throw new Error(
            `Failed to get contract instance for "${contractName}" at ${address}: ${error.message}`
        );
    }
}

/**
 * Get all contract instances
 * @param {Object} hre - Hardhat runtime environment
 * @returns {Promise<Object>} Object with all contract instances
 */
async function getAllContractInstances(hre) {
    const deployments = loadDeployment();
    const contractNames = [
        'LineageVerifier',
        'Groth16Verifier',
        'EpochManager',
        'RateLimiter',
        'AuthorizationVerifier',
        'PolicyRegistry',
        'BatchVerifier'
    ];

    const instances = {};

    for (const contractName of contractNames) {
        try {
            instances[contractName] = await getContractInstance(hre, contractName);
        } catch (error) {
            console.warn(`Warning: Could not load ${contractName}:`, error.message);
        }
    }

    return instances;
}

/**
 * Get contract address by name
 * @param {string} contractName - Name of the contract
 * @returns {string} Contract address
 */
function getContractAddress(contractName) {
    const deployments = loadDeployment();
    const address = deployments[contractName];
    
    if (!address) {
        throw new Error(`Contract "${contractName}" not found in deployments`);
    }
    
    return address;
}

/**
 * Get all contract addresses
 * @returns {Object} Object with all contract addresses
 */
function getAllContractAddresses() {
    const deployments = loadDeployment();
    return {
        LineageVerifier: deployments.LineageVerifier,
        Groth16Verifier: deployments.Groth16Verifier,
        EpochManager: deployments.EpochManager,
        RateLimiter: deployments.RateLimiter,
        AuthorizationVerifier: deployments.AuthorizationVerifier,
        PolicyRegistry: deployments.PolicyRegistry,
        BatchVerifier: deployments.BatchVerifier
    };
}

/**
 * Get deployment metadata
 * @returns {Object} Metadata about the deployment
 */
function getDeploymentMetadata() {
    const deployments = loadDeployment();
    return {
        network: deployments.network,
        chainId: deployments.chainId,
        deployer: deployments.deployer,
        timestamp: deployments.timestamp,
        genesisStateHash: deployments.genesisStateHash,
        genesisLineageCommitment: deployments.genesisLineageCommitment,
        policyRoot: deployments.policyRoot,
        transitionCount: deployments.transitionCount
    };
}

module.exports = {
    loadDeployment,
    getContractInstance,
    getAllContractInstances,
    getContractAddress,
    getAllContractAddresses,
    getDeploymentMetadata
};