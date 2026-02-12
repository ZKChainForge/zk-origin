import { buildPoseidon } from "circomlibjs";

// Origin class enum
export enum Origin {
    Genesis = 0,
    User = 1,
    Admin = 2,
    Bridge = 3
}

export const OriginNames = ["Genesis", "User", "Admin", "Bridge"];

// Policy check
export function isPolicyAllowed(prevOrigin: Origin, newOrigin: Origin, prevDepth: number): boolean {
    if (prevOrigin === Origin.User && newOrigin === Origin.Admin) return false;
    if (prevOrigin === Origin.User && newOrigin === Origin.Bridge) return false;
    if (prevOrigin === Origin.Bridge && newOrigin === Origin.Admin) return false;
    if (prevOrigin === Origin.Bridge && newOrigin === Origin.Bridge) return false;
    if (prevDepth > 0 && newOrigin === Origin.Genesis) return false;
    return true;
}

// Generate random hash
export function randomHash(): bigint {
    const bytes = new Uint8Array(32);
    for (let i = 0; i < 32; i++) {
        bytes[i] = Math.floor(Math.random() * 256);
    }
    return BigInt("0x" + Array.from(bytes).map(b => b.toString(16).padStart(2, "0")).join(""));
}

// Poseidon hash wrapper
let poseidonInstance: any = null;

export async function getPoseidon() {
    if (!poseidonInstance) {
        poseidonInstance = await buildPoseidon();
    }
    return poseidonInstance;
}

export async function poseidonHash(inputs: bigint[]): Promise<bigint> {
    const poseidon = await getPoseidon();
    const result = poseidon(inputs);
    return poseidon.F.toObject(result);
}

// Genesis lineage computation
export async function genesisLineage(stateHash: bigint): Promise<bigint> {
    return poseidonHash([stateHash, BigInt(Origin.Genesis), BigInt(0)]);
}

// Transition hash computation
export async function transitionHash(
    prevState: bigint,
    newState: bigint,
    origin: Origin,
    timestamp: number
): Promise<bigint> {
    return poseidonHash([prevState, newState, BigInt(origin), BigInt(timestamp)]);
}

// Lineage commitment update
export async function updateLineage(
    prevLineage: bigint,
    txHash: bigint,
    newDepth: number
): Promise<bigint> {
    return poseidonHash([prevLineage, txHash, BigInt(newDepth)]);
}

// Format bigint for display
export function formatHash(hash: bigint, length: number = 16): string {
    const hex = hash.toString(16).padStart(64, "0");
    return "0x" + hex.slice(0, length) + "...";
}