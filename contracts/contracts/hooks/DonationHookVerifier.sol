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
    uint256 constant alphax  = 16305772100422889692794511674290143751225861801915870020412959517476065883774;
    uint256 constant alphay  = 1730453349516491842294927247674147995798799462436198607788733113746980402035;
    uint256 constant betax1  = 5985437379609465049698510571051512335741716724957524660310231413608825992584;
    uint256 constant betax2  = 4658514544878587724642349759783413593226081419400818633586981111353050967834;
    uint256 constant betay1  = 758484219893823974953788657650306506081815595270531940013459663273475221008;
    uint256 constant betay2  = 1388030110068379802934582490864738593263202465719605230255284783451091147901;
    uint256 constant gammax1 = 11559732032986387107991004021392285783925812861821192530917403151452391805634;
    uint256 constant gammax2 = 10857046999023057135944570762232829481370756359578518086990519993285655852781;
    uint256 constant gammay1 = 4082367875863433681332203403145435568316851327593401208105741076214120093531;
    uint256 constant gammay2 = 8495653923123431417604973247489272438418190587263600148770280649306958101930;
    uint256 constant deltax1 = 10127685554540646542496476828373290116147589262199101681473026206956180702834;
    uint256 constant deltax2 = 4668608368583410493135915345665674876946852510253034874839111868323584242452;
    uint256 constant deltay1 = 19334932780912270577218500215385676426835880617629301884667327184470452076689;
    uint256 constant deltay2 = 13018457515647873718036715753026651840662356729080242572337639184888629850116;

    
    uint256 constant IC0x = 3592380443101730563668620028845326103182884479363945843692578870974895366396;
    uint256 constant IC0y = 21431012473019922625684088234820427102811463418352059185681471656712312017697;
    
    uint256 constant IC1x = 14807812771767303563352314577629071129123571240158130767561509399555545246712;
    uint256 constant IC1y = 7122873561962230680193810038299923940819293901034526811271810430345014607182;
    
    uint256 constant IC2x = 4175919746426033980392991648259665929430505691397934400702378211170201242831;
    uint256 constant IC2y = 7231843396509283950464676430371009966712695417959868523252310476802892916179;
    
    uint256 constant IC3x = 4052784484331850508844927577579311974214939723144402319983220515326357224269;
    uint256 constant IC3y = 413370077961259205920325913763289016111058741893483012647720119460553562018;
    
    uint256 constant IC4x = 896174570219622030744303163400431557638758346092092979074827233954167240301;
    uint256 constant IC4y = 11694657410253915252856134519892623871000395424575573577850334390011067924716;
    
    uint256 constant IC5x = 8744922855792935698580442966593757313131021920506824504504808769412748168783;
    uint256 constant IC5y = 17776105786632764373618714895635799378334362883373902270642471909490086998774;
    
    uint256 constant IC6x = 3756795352840059974329284806899076543627365027116363304067347784113788311892;
    uint256 constant IC6y = 20971340963036637176804972347926503752786047065972239397550389222749732092228;
    
    uint256 constant IC7x = 21024407940621762325132803255904058311309053002871312834312562441681361439900;
    uint256 constant IC7y = 17857030038873089382816763557043206445377080617578656594745899633075780992447;
    
    uint256 constant IC8x = 16347536718840355019679519261480857686017767207298669555972256292607332944979;
    uint256 constant IC8y = 19900482724297269982444586770197234770607398893435668213001878692659879027419;
    
    uint256 constant IC9x = 2203316447612787679843270090506271717286499447760736689699502550158151141981;
    uint256 constant IC9y = 5565531849988045642570922198310619588375475231815158210840201368294966609969;
    
    uint256 constant IC10x = 16283522200162870011594775578326600845171521968578974609091801165528921692026;
    uint256 constant IC10y = 5016521983810431062684702875236284380237329268295373813553642987503935224173;
    
    uint256 constant IC11x = 21622878320789979293310543311589390591472331387672971927029186044517964165539;
    uint256 constant IC11y = 10386057381587085476188529392810254227858020185317469888731720782411064723496;
    
    uint256 constant IC12x = 15537771502724061717647668618190182527518206601881655612995794346055374894610;
    uint256 constant IC12y = 14212234821729961992174239284958295683459293178894608569340836259864612092629;
    
 
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
