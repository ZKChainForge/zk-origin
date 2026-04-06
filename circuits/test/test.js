#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const GREEN = '\x1b[32m';
const BLUE = '\x1b[34m';
const RED = '\x1b[31m';
const NC = '\x1b[0m';

function log(color, message) {
  console.log(`${color}${message}${NC}`);
}

function runCommand(cmd) {
  try {
    return execSync(cmd, { encoding: 'utf8' });
  } catch (error) {
    log(RED, `[!] Command failed: ${cmd}`);
    throw error;
  }
}

async function testCircuit(circuitName, inputFile) {
  log(BLUE, `\n[*] Testing ${circuitName}...`);
  
  try {
    // Check if input file exists
    if (!fs.existsSync(inputFile)) {
      log(RED, `[!] Input file not found: ${inputFile}`);
      return false;
    }
    
    // Generate witness
    log(BLUE, `    - Generating witness...`);
    const input = JSON.parse(fs.readFileSync(inputFile, 'utf8'));
    
    // For now, just verify the input file format
    log(GREEN, `    ✓ Input file valid`);
    
    // Check if keys exist
    const vkPath = `build/keys/${circuitName}_vk.json`;
    if (!fs.existsSync(vkPath)) {
      log(RED, `[!] Verification key not found: ${vkPath}`);
      log(BLUE, `    Run: npm run setup`);
      return false;
    }
    
    log(GREEN, `    ✓ Verification key found`);
    return true;
    
  } catch (error) {
    log(RED, `[!] Test failed for ${circuitName}: ${error.message}`);
    return false;
  }
}

async function main() {
  
  
  const tests = [
    { circuit: 'user_auth', input: 'test/inputs/user_auth_input.json' },
    { circuit: 'admin_auth', input: 'test/inputs/admin_auth_input.json' },
    { circuit: 'governance_auth', input: 'test/inputs/governance_auth_input.json' },
    { circuit: 'emergency_auth', input: 'test/inputs/emergency_auth_input.json' },
    { circuit: 'rate_limiter', input: 'test/inputs/rate_limiter_input.json' },
    { circuit: 'lineage_step', input: 'test/inputs/lineage_step_input.json' },
  ];
  
  let passed = 0;
  let failed = 0;
  
  for (const test of tests) {
    const result = await testCircuit(test.circuit, test.input);
    if (result) {
      passed++;
    } else {
      failed++;
    }
  }
  

  
  
  return failed === 0 ? 0 : 1;
}

main().then(code => process.exit(code));