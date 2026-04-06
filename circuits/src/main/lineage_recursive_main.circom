pragma circom 2.1.0;
include "../core/lineage_recursive.circom";
component main {public [prevStateHash, newStateHash, epochId, prevOriginClass, newOriginClass]} = RecursiveLineageStep();
