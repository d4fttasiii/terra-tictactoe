import { LCDClient, MsgExecuteContract, Fee, Coin } from "@terra-money/terra.js";
import { contractAdress } from "./address";

// ==== utils ====

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const until = Date.now() + 1000 * 60 * 60;
const untilInterval = Date.now() + 1000 * 60;

const _exec = (msg, fee = new Fee(200000, { uluna: 10000 })) =>
  async (wallet) => {
    const lcd = new LCDClient({
      URL: wallet.network.lcd,
      chainID: wallet.network.chainID,
    });

    const { result } = await wallet.post({
      fee,
      msgs: [
        new MsgExecuteContract(
          wallet.walletAddress,
          contractAdress(wallet),
          msg,

        ),
      ],
    });

    while (true) {
      try {
        return await lcd.tx.txInfo(result.txhash);
      } catch (e) {
        if (Date.now() < untilInterval) {
          await sleep(500);
        } else if (Date.now() < until) {
          await sleep(1000 * 10);
        } else {
          throw new Error(
            `Transaction queued. To verify the status, please check the transaction hash: ${result.txhash}`
          );
        }
      }
    }
  };

const _execAndSend = (msg, bet, fee = new Fee(2000000, { uluna: 25000 })) =>
  async (wallet) => {
    const lcd = new LCDClient({
      URL: wallet.network.lcd,
      chainID: wallet.network.chainID,
    });

    const { result } = await wallet.post({
      fee,
      gasPrices: { uluna: 0.01133 },
      msgs: [
        new MsgExecuteContract(
          wallet.walletAddress,
          contractAdress(wallet),
          msg,
          [new Coin('uluna', bet.toString())]
        ),
      ],
    });

    while (true) {
      try {
        return await lcd.tx.txInfo(result.txhash);
      } catch (e) {
        if (Date.now() < untilInterval) {
          await sleep(500);
        } else if (Date.now() < until) {
          await sleep(1000 * 10);
        } else {
          throw new Error(
            `Transaction queued. To verify the status, please check the transaction hash: ${result.txhash}`
          );
        }
      }
    }
  };

// ==== execute contract ====

export const createGame = async (wallet, bet) => {
  _execAndSend({
    create_game: {}
  }, bet)(wallet);
};

export const joinGame = async (wallet, gameId, bet) => {
  _execAndSend({
    join_game: {
      game_id: gameId
    }
  }, bet)(wallet);
};

export const makeMove = async (wallet, gameId, x, y) => {
  _exec({
    make_move: {
      game_id: gameId,
      x: x,
      y: y,
    }
  })(wallet);
};

export const withdrawPrice = async (wallet, gameId) => {
  _exec({
    withdraw_price: {
      game_id: gameId,
    }
  })(wallet);
};

// export const reset = async (wallet, count) =>
//   _exec({ reset: { count } })(wallet);
