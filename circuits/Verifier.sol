// SPDX-License-Identifier: GPL-3.0
/*
    Copyright 2021 0KIMS association.

    This file is generated with [snarkJS](https://github.com/iden3/snarkjs).

    snarkJS is a free software: you can redistribute it and/or modify it
    under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    snarkJS is distributed in the hope that it will be useful, but WITHOUT
    ANY WARRANTY; without even the implied warranty of MERCHANTABILITY
    or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public
    License for more details.

    You should have received a copy of the GNU General Public License
    along with snarkJS. If not, see <https://www.gnu.org/licenses/>.
*/

pragma solidity >=0.7.0 <0.9.0;

contract Groth16Verifier {
    // Scalar field size
    uint256 constant r    = 21888242871839275222246405745257275088548364400416034343698204186575808495617;
    // Base field size
    uint256 constant q   = 21888242871839275222246405745257275088696311157297823662689037894645226208583;

    // Verification Key data
    uint256 constant alphax  = 2728015306398326184234426255357907167480910937462709935241722928432452471647;
    uint256 constant alphay  = 14249575676048669837605104270702010629882452652202633789635594360710549150502;
    uint256 constant betax1  = 19574968418998409440684644430210175660171902693979351327878123375226823973876;
    uint256 constant betax2  = 12589895217676193004254079486370076290164844752973668141327199716550109271080;
    uint256 constant betay1  = 15003282238522861991687663771193427363001158484668266042953619834023677576802;
    uint256 constant betay2  = 1759922341476224821133404857534317002833556932337592028948621257171832993373;
    uint256 constant gammax1 = 11559732032986387107991004021392285783925812861821192530917403151452391805634;
    uint256 constant gammax2 = 10857046999023057135944570762232829481370756359578518086990519993285655852781;
    uint256 constant gammay1 = 4082367875863433681332203403145435568316851327593401208105741076214120093531;
    uint256 constant gammay2 = 8495653923123431417604973247489272438418190587263600148770280649306958101930;
    uint256 constant deltax1 = 6246370924699466323196732651127730469311665130851750036074046036411868367932;
    uint256 constant deltax2 = 7062806211113131476939306751657271227229743146762821793505789145049316952359;
    uint256 constant deltay1 = 17180254206269464719165399235679579484719672789776673295352873382156887379547;
    uint256 constant deltay2 = 21524157087387687903058272221818484839655680170948321797196358430348526773550;

    
    uint256 constant IC0x = 21824899272174434144044103172786160811573572684638642538903541448230837381669;
    uint256 constant IC0y = 19350216636435696363371571251856166601169307403463402307679467967787734494458;
    
    uint256 constant IC1x = 17175683472874918964763792736849101231128758549483577075078531849838297824141;
    uint256 constant IC1y = 1683335624455250084953685372539258794507737341840851307791913786662942756408;
    
    uint256 constant IC2x = 12634581856969424538223641531095318430446219037665097488988921185652870650628;
    uint256 constant IC2y = 6195999304329029244010100569820216549646224417777408375713988144011557178714;
    
    uint256 constant IC3x = 14896572979228522942975652076066177025319492853107775395480757210181348280895;
    uint256 constant IC3y = 18238454646326983068964293062450204649838761178048068727872529105053903522085;
    
    uint256 constant IC4x = 330135871008426526615418904336182639713924447958081066896445762250637214111;
    uint256 constant IC4y = 13245058896047726231466025568166835647434332139440738229165298809841261439296;
    
    uint256 constant IC5x = 19472426177121758226838999878704037499107056496703608406958539077592866258640;
    uint256 constant IC5y = 6997905921507260935104315505761280743816079331245538321701756206027138439679;
    
    uint256 constant IC6x = 18441957667855365933939962543135365602810159732650680814714143279436552626001;
    uint256 constant IC6y = 11176306809567239487598743252178500173148999726444261243147076650461055834338;
    
    uint256 constant IC7x = 10475976800732959164304250538111650003537178718429819375351900262750497076399;
    uint256 constant IC7y = 18419667335359719320682136366284227224826211727370244579080681085673767869944;
    
    uint256 constant IC8x = 16666341415718644473219270983681502361947048657486023299845033566354571737253;
    uint256 constant IC8y = 20926839735997381634914635242843723791350128281456647405411329620813545060278;
    
    uint256 constant IC9x = 17683599079632958417516523558527017125676576894477011469801453287599074345606;
    uint256 constant IC9y = 10738927050675868561974948285963011580508375199708973813945066358792826969674;
    
    uint256 constant IC10x = 14744390988851682767355809028027434724348610013983306364509232999721357147979;
    uint256 constant IC10y = 19419387468652545279215297973471080783589285285999867993591977330685768632519;
    
    uint256 constant IC11x = 11345955769189860025161977302905400115714441119897574678650362668584339784128;
    uint256 constant IC11y = 2119259671467286388729659954998045399289479119473312949087262154989125119327;
    
    uint256 constant IC12x = 18256355647678146328072022413049532723484556520761170213264166099805400780669;
    uint256 constant IC12y = 3331299265780964173160429402114502513905043661865699154858913058243683505014;
    
 
    // Memory data
    uint16 constant pVk = 0;
    uint16 constant pPairing = 128;

    uint16 constant pLastMem = 896;

    function verifyProof(uint[2] calldata _pA, uint[2][2] calldata _pB, uint[2] calldata _pC, uint[12] calldata _pubSignals) public view returns (bool) {
        assembly {
            function checkField(v) {
                if iszero(lt(v, r)) {
                    mstore(0, 0)
                    return(0, 0x20)
                }
            }
            
            // G1 function to multiply a G1 value(x,y) to value in an address
            function g1_mulAccC(pR, x, y, s) {
                let success
                let mIn := mload(0x40)
                mstore(mIn, x)
                mstore(add(mIn, 32), y)
                mstore(add(mIn, 64), s)

                success := staticcall(sub(gas(), 2000), 7, mIn, 96, mIn, 64)

                if iszero(success) {
                    mstore(0, 0)
                    return(0, 0x20)
                }

                mstore(add(mIn, 64), mload(pR))
                mstore(add(mIn, 96), mload(add(pR, 32)))

                success := staticcall(sub(gas(), 2000), 6, mIn, 128, pR, 64)

                if iszero(success) {
                    mstore(0, 0)
                    return(0, 0x20)
                }
            }

            function checkPairing(pA, pB, pC, pubSignals, pMem) -> isOk {
                let _pPairing := add(pMem, pPairing)
                let _pVk := add(pMem, pVk)

                mstore(_pVk, IC0x)
                mstore(add(_pVk, 32), IC0y)

                // Compute the linear combination vk_x
                
                g1_mulAccC(_pVk, IC1x, IC1y, calldataload(add(pubSignals, 0)))
                
                g1_mulAccC(_pVk, IC2x, IC2y, calldataload(add(pubSignals, 32)))
                
                g1_mulAccC(_pVk, IC3x, IC3y, calldataload(add(pubSignals, 64)))
                
                g1_mulAccC(_pVk, IC4x, IC4y, calldataload(add(pubSignals, 96)))
                
                g1_mulAccC(_pVk, IC5x, IC5y, calldataload(add(pubSignals, 128)))
                
                g1_mulAccC(_pVk, IC6x, IC6y, calldataload(add(pubSignals, 160)))
                
                g1_mulAccC(_pVk, IC7x, IC7y, calldataload(add(pubSignals, 192)))
                
                g1_mulAccC(_pVk, IC8x, IC8y, calldataload(add(pubSignals, 224)))
                
                g1_mulAccC(_pVk, IC9x, IC9y, calldataload(add(pubSignals, 256)))
                
                g1_mulAccC(_pVk, IC10x, IC10y, calldataload(add(pubSignals, 288)))
                
                g1_mulAccC(_pVk, IC11x, IC11y, calldataload(add(pubSignals, 320)))
                
                g1_mulAccC(_pVk, IC12x, IC12y, calldataload(add(pubSignals, 352)))
                

                // -A
                mstore(_pPairing, calldataload(pA))
                mstore(add(_pPairing, 32), mod(sub(q, calldataload(add(pA, 32))), q))

                // B
                mstore(add(_pPairing, 64), calldataload(pB))
                mstore(add(_pPairing, 96), calldataload(add(pB, 32)))
                mstore(add(_pPairing, 128), calldataload(add(pB, 64)))
                mstore(add(_pPairing, 160), calldataload(add(pB, 96)))

                // alpha1
                mstore(add(_pPairing, 192), alphax)
                mstore(add(_pPairing, 224), alphay)

                // beta2
                mstore(add(_pPairing, 256), betax1)
                mstore(add(_pPairing, 288), betax2)
                mstore(add(_pPairing, 320), betay1)
                mstore(add(_pPairing, 352), betay2)

                // vk_x
                mstore(add(_pPairing, 384), mload(add(pMem, pVk)))
                mstore(add(_pPairing, 416), mload(add(pMem, add(pVk, 32))))


                // gamma2
                mstore(add(_pPairing, 448), gammax1)
                mstore(add(_pPairing, 480), gammax2)
                mstore(add(_pPairing, 512), gammay1)
                mstore(add(_pPairing, 544), gammay2)

                // C
                mstore(add(_pPairing, 576), calldataload(pC))
                mstore(add(_pPairing, 608), calldataload(add(pC, 32)))

                // delta2
                mstore(add(_pPairing, 640), deltax1)
                mstore(add(_pPairing, 672), deltax2)
                mstore(add(_pPairing, 704), deltay1)
                mstore(add(_pPairing, 736), deltay2)


                let success := staticcall(sub(gas(), 2000), 8, _pPairing, 768, _pPairing, 0x20)

                isOk := and(success, mload(_pPairing))
            }

            let pMem := mload(0x40)
            mstore(0x40, add(pMem, pLastMem))

            // Validate that all evaluations ∈ F
            
            checkField(calldataload(add(_pubSignals, 0)))
            
            checkField(calldataload(add(_pubSignals, 32)))
            
            checkField(calldataload(add(_pubSignals, 64)))
            
            checkField(calldataload(add(_pubSignals, 96)))
            
            checkField(calldataload(add(_pubSignals, 128)))
            
            checkField(calldataload(add(_pubSignals, 160)))
            
            checkField(calldataload(add(_pubSignals, 192)))
            
            checkField(calldataload(add(_pubSignals, 224)))
            
            checkField(calldataload(add(_pubSignals, 256)))
            
            checkField(calldataload(add(_pubSignals, 288)))
            
            checkField(calldataload(add(_pubSignals, 320)))
            
            checkField(calldataload(add(_pubSignals, 352)))
            

            // Validate all evaluations
            let isValid := checkPairing(_pA, _pB, _pC, _pubSignals, pMem)

            mstore(0, isValid)
             return(0, 0x20)
         }
     }
 }
