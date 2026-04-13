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
    uint256 constant alphax  = 16951762493308411018081532313238858298965210628613499714136650857710765155373;
    uint256 constant alphay  = 14348209960918647744087907981642269133769183276962330994890595734800543089724;
    uint256 constant betax1  = 19933559818860856407498993711195345555766641580745064852219748859555193879867;
    uint256 constant betax2  = 20978488023541864571345615990663205925866340980295880318216576390881460851066;
    uint256 constant betay1  = 13948412975750552610214983670705072761106961201517842126666247117988004413290;
    uint256 constant betay2  = 4273346333683026573868810913881008570504345249689767469506114538665320694189;
    uint256 constant gammax1 = 11559732032986387107991004021392285783925812861821192530917403151452391805634;
    uint256 constant gammax2 = 10857046999023057135944570762232829481370756359578518086990519993285655852781;
    uint256 constant gammay1 = 4082367875863433681332203403145435568316851327593401208105741076214120093531;
    uint256 constant gammay2 = 8495653923123431417604973247489272438418190587263600148770280649306958101930;
    uint256 constant deltax1 = 4283440200128351529273307883273088386790521839900924480107113159164494146822;
    uint256 constant deltax2 = 16397138887233637337832183975083429374222260877733942060908601351200095585507;
    uint256 constant deltay1 = 18476798423537685244885886972889943546605457458701911516746092378483341155391;
    uint256 constant deltay2 = 3762160640464664557597634658300803094696179378076565053971241314804550751001;

    
    uint256 constant IC0x = 7297217918702457646773800571459352439726003225280262072579321219986877005072;
    uint256 constant IC0y = 19615817459717233388428349308729678964877062470418849937492942236988294228600;
    
    uint256 constant IC1x = 12463658802922869968222691116035553230685808717342503254148929419864082435163;
    uint256 constant IC1y = 9528245537192993442920512660836413953819672802579898427701547350533575934400;
    
    uint256 constant IC2x = 12526161412162419349115730286528818609036441485036894604839526347432509177134;
    uint256 constant IC2y = 20515553512716619911848565992978867016755319300644123449969631269377826066599;
    
    uint256 constant IC3x = 13150809132750819793347864156713289070829296194754464599596405937653438187017;
    uint256 constant IC3y = 3007901756064664336271369564288635783486562761654964716260414227943880672063;
    
    uint256 constant IC4x = 7751209773678125488111939084922816923504248209587018037150198687543560024390;
    uint256 constant IC4y = 13019675957204125853409961341640096141136003234934696503286173959279291708574;
    
    uint256 constant IC5x = 8618153902141848461608913086503658602852455318967563313947004742679177395132;
    uint256 constant IC5y = 9926296250422284542305355068236319356593118778762695119361679352817323693992;
    
    uint256 constant IC6x = 9579068298250994685582390856725916871419595099430012062957949522642018856129;
    uint256 constant IC6y = 17919405387393871991237512197710726729118454247967262780219894521837927150119;
    
    uint256 constant IC7x = 13181732522091604646930103620437554179533878420581878168251806479830295999841;
    uint256 constant IC7y = 19649284172895995880580081797005049265684594173947954433890559009634079371840;
    
    uint256 constant IC8x = 1568221473714235588913540167217770136969426335897762644714503711332622384599;
    uint256 constant IC8y = 17592570988704849544976097367612224331519504670405953151821121672114296333453;
    
    uint256 constant IC9x = 19668169578854773105743911405015578320639519553529646890926334159100467298323;
    uint256 constant IC9y = 9280390486843695970338112036538503867767434223856690873322361789295766975046;
    
    uint256 constant IC10x = 19398487986724437998886640919448174249934968180656957428788925524781915871261;
    uint256 constant IC10y = 5796557978312393628235233985657678069778601825858763485375924657777665867340;
    
    uint256 constant IC11x = 16081421588861511733154511862667655759035598638306431049876245552773212524631;
    uint256 constant IC11y = 14285773168474259685502981179448405956226542051215597803315192034546384409649;
    
    uint256 constant IC12x = 12258553337645922218400053252418597893723136433960507215420987753010799596513;
    uint256 constant IC12y = 11559402275277207485952415660450718485727288163872566314801830586207626751398;
    
 
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
