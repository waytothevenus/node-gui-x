import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { exit } from "@tauri-apps/plugin-process";
import { AiOutlineCopy } from "react-icons/ai";
import { listen } from "@tauri-apps/api/event";
import { RiDeleteBinLine } from "react-icons/ri";
import { notify } from "../utils/util";

import { AccountType, ConsoleCommand, WalletInfo } from "../types/Types";

const Console = (props: {
  currentAccount: AccountType | undefined;
  currentAccountId: number;
  currentWallet: WalletInfo | undefined;
}) => {
  const [text, setText] = useState("");
  const [command, setCommand] = useState("");
  const [commandHistory, setCommandHistory] = useState<string[]>([]);
  const handleDelete = () => {
    setText("");
  };
  const handleCopy = () => {
    navigator.clipboard.writeText(text);
  };

  const handleExit = async () => {
    try {
      await invoke("shutdown_wrapper");
    } catch (error) {
      console.error("Error shutting down node", error);
    }
    await exit();
  };

  const handleSendCommand = async () => {
    try {
      console.log("handleSendCommand is called");
      const unsubscribe = await listen("ConsoleResponse", (event) => {
        setCommandHistory((history) => [...history, command]);
        const consoleResult = event.payload as ConsoleCommand;
        console.log("console response is: ", event.payload);
        if (typeof consoleResult === "string") {
          setText((text) => text + "\n" + command + "\n" + consoleResult);
        } else if ("ClearScreen" in consoleResult) {
          setText("");
        } else if ("PrintHistory" in consoleResult) {
          setText(
            (text) => text + "\n" + command + "\n" + commandHistory.join("\n")
          );
        } else if ("ClearHistory" in consoleResult) {
          setCommandHistory([]);
        } else if ("Exit" in consoleResult) {
          handleExit();
        } else if ("SetStatus" in consoleResult) {
          setText(
            (text) =>
              text +
              "\n" +
              command +
              "\n" +
              consoleResult.SetStatus.print_message
          );
        } else if ("Print" in consoleResult) {
          setText((text) => text + "\n" + command + "\n" + consoleResult.Print);
        } else {
          setText(
            (text) =>
              text + "\n" + command + "\n" + JSON.stringify(consoleResult)
          );
        }
        setCommand("");
        unsubscribe();
      });
      await invoke("handle_console_command_wrapper", {
        request: {
          wallet_id: props.currentWallet?.wallet_id
            ? props.currentWallet?.wallet_id
            : 0,
          account_id: props.currentAccountId,
          command: command,
        },
      });
    } catch (error) {
      notify(new String(error).toString(), "error");
    }
  };

  const handleKeyDown = (event: React.KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Enter") {
      handleSendCommand();
    }
  };
  return (
    <div className="container border-1px border-gray-100 space-y-4 pt-1">
      <div className="mt-10 p-4 border border-gray-300 rounded-lg ">
        <div className="flex justify-between items-center">
          <h2 className="text-lg font-bold mb-4">Console</h2>
          <div className="flex items-center">
            <div className="w-px h-6 bg-gray-400 mx-2"></div>
            <div className="flex justify-between items-center space-x-4">
              <button
                onClick={handleCopy}
                className="flex items-center justify-center p-0 bg-transparent border-none shadow-none focus:outline-none"
              >
                <AiOutlineCopy />
              </button>
              <button
                onClick={handleDelete}
                className="flex items-center justify-center p-0 bg-transparent border-none shadow-none focus:outline-none"
              >
                <RiDeleteBinLine />
              </button>
            </div>
          </div>
        </div>
        <textarea
          className="w-full h-32 border border-gray-300 rounded-lg p-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
          value={text}
          readOnly
        />
      </div>
      <div className="justify-between flex rounded-lg space-x-4 border border-gray-100">
        <input
          className=" w-full"
          placeholder="Type here ..."
          value={command}
          onChange={(e) => setCommand(e.target.value)}
          onKeyDown={(event) => handleKeyDown(event)}
        />
        <button
          onClick={handleSendCommand}
          className="w-[5rem] py-1 px-4 rounded-lg bg-[#69EE96] text-[#000000] hover:text-[#69EE96] hover:bg-black "
        >
          Send
        </button>
      </div>
    </div>
  );
};

export default Console;
