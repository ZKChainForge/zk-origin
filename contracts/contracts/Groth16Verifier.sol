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
    uint256 constant deltax1 = 1208327763519737123398765438695629953747668754458696333773032635185611318829;
    uint256 constant deltax2 = 15751922124273531746967517298779924511761736313073000072463830253111122062440;
    uint256 constant deltay1 = 7229090853923595657486005072287258707259828630029465440028493862817092745427;
    uint256 constant deltay2 = 21206131568992495260801636089169453540641010172635369285161829493191353304047;

    
    uint256 constant IC0x = 11921780030809391704199201758591288661385257890882493059502001963044290172988;
    uint256 constant IC0y = 16276103420667347413709391297067412986329511130827728711696040101766350663777;
    
    uint256 constant IC1x = 11169530450583020708249241516706096875749604153959661903657859530412042533999;
    uint256 constant IC1y = 21532419720795407920498880301458504654971975856496533593539964900052656662856;
    
    uint256 constant IC2x = 7046644684585838124435622394783417876538281600118141351563608965369419366644;
    uint256 constant IC2y = 8421345242183036091669211977977753440176271853734707857304570145335476773149;
    
    uint256 constant IC3x = 5452768238178192496547989685374423109224816479068907601552788540382379538866;
    uint256 constant IC3y = 19808821719730668951928438090007333966309834134244266911781653833300888734182;
    
    uint256 constant IC4x = 17594900140963462930260995253592909028047236811073625083523065927128936785741;
    uint256 constant IC4y = 19560562732109134615465380788321696474367001246756507845210386257367746681302;
    
    uint256 constant IC5x = 9823708462757748718202197527544333974565073486725576246863750732511032163607;
    uint256 constant IC5y = 11897803673666440193802469311349703309848528492301633063872510713910198431738;
    
    uint256 constant IC6x = 1959064513755164649956683817443960745681207405054001392684760070706780211102;
    uint256 constant IC6y = 9573876163311405343177359168818112618511791595513512914421365298167086604781;
    
    uint256 constant IC7x = 21283929660620477728392028663887260101232841099504665914786208532319914908910;
    uint256 constant IC7y = 14909939417008920066598708824918248690732112351308856704279947962528814080459;
    
    uint256 constant IC8x = 10062117577351611931957617041381472937697849529529831838565608322527715631009;
    uint256 constant IC8y = 16751494945101619683709942948239685715790906348702555829305997295509619333418;
    
    uint256 constant IC9x = 3185602096968814256832036552457615524393493780332264318267222197686336983804;
    uint256 constant IC9y = 21386960954428066191795059496401144034707564743391188241573106225861194315802;
    
    uint256 constant IC10x = 16319573808775343053601866785511896508830411245889075489265135757703845218801;
    uint256 constant IC10y = 15334017772842626794483244689007960878495175176072997373388647637510272992298;
    
    uint256 constant IC11x = 16235829283661888447829131314416702054492184546698812053780398067448607948335;
    uint256 constant IC11y = 14259574658805973410723287574073440167480845240661017394485980031552723008322;
    
    uint256 constant IC12x = 21684364453189192640435819227707929345636238928652281739972779737180046478869;
    uint256 constant IC12y = 2667954171237029867216144467064012134650670266219332284506226333617264752107;
    
 
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
