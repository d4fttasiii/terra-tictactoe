import { LCDClient } from '@terra-money/terra.js'
import { contractAdress } from './address'

export const getGames = async (wallet) => {
  const lcd = new LCDClient({
    URL: wallet.network.lcd,
    chainID: wallet.network.chainID,
  });

  return lcd.wasm.contractQuery(contractAdress(wallet), { get_games: {} });
};

export const getGame = async (wallet, gameId) => {
  const lcd = new LCDClient({
    URL: wallet.network.lcd,
    chainID: wallet.network.chainID,
  });

  return lcd.wasm.contractQuery(contractAdress(wallet), { get_game_by_id: { id: gameId } });
};

export const getGamesByAddress = async (wallet) => {
  const lcd = new LCDClient({
    URL: wallet.network.lcd,
    chainID: wallet.network.chainID,
  });

  return lcd.wasm.contractQuery(contractAdress(wallet), { get_games_by_address: { address: wallet.terraAddress } });
};

export const getWinCount = async (wallet) => {
  const lcd = new LCDClient({
    URL: wallet.network.lcd,
    chainID: wallet.network.chainID,
  });

  return lcd.wasm.contractQuery(contractAdress(wallet), { get_leaderboard: { address: wallet.terraAddress } });
}