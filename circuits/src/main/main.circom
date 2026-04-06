pragma circom 2.1.0;
include "../core/lineage_step.circom";
component main {public [prevStateHash, newStateHash, epochId, prevOriginClass, newOriginClass]} = LineageStep();
