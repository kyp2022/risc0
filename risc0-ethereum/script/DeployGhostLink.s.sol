// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import {Script} from "forge-std/Script.sol";
import {console2} from "forge-std/console2.sol";
import {GhostLinkSBT} from "../test/GhostLinkSBT.sol";

contract DeployGhostLink is Script {
    function run() external {
        // 1. 获取部署者的私钥 (需要你在环境变量中设置 PRIVATE_KEY)
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");

        // 2. 设置你的 Image ID
        // 重要：这里必须填入你 Rust 项目生成的 Image ID！
        // 运行 host 程序时会在控制台打印出来，例如：
        // 🔑 GUEST IMAGE ID: 0x...
        // 请将下面的值替换为你实际运行 host 得到的 Image ID
        bytes32 imageId = 0xd9d35df9593604b2e0a6f0a5d7a3d08fbf8aec70dca9b6b362058b20f00204b0;

        vm.startBroadcast(deployerPrivateKey);

        // 3. 部署 GhostLinkSBT
        // 构造函数只接受 imageId，Verifier 会在合约内自动部署
        GhostLinkSBT sbt = new GhostLinkSBT(imageId);

        console2.log("GhostLinkSBT deployed at:", address(sbt));
        console2.log("Verifier deployed at:", address(sbt.verifier()));

        vm.stopBroadcast();
    }
}
