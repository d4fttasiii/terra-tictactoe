import './App.css';

import { useEffect, useState } from 'react';
import {
  useWallet,
  useConnectedWallet,
  WalletStatus,
} from '@terra-money/wallet-provider';

import * as execute from './contract/execute';
import * as query from './contract/query';
import { ConnectWallet } from './components/ConnectWallet';

import InputLabel from '@mui/material/InputLabel';
import Input from '@mui/material/Input';
import InputAdornment from '@mui/material/InputAdornment';
import AppBar from '@mui/material/AppBar';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Toolbar from '@mui/material/Toolbar';
import Typography from '@mui/material/Typography';
import Container from '@mui/material/Container';
import { ThemeProvider, createTheme } from '@mui/material/styles';
import Grid from '@mui/material/Grid';
import FormControl from '@mui/material/FormControl';
import CloseIcon from '@mui/icons-material/Close';
import CircleIcon from '@mui/icons-material/Circle';
import CheckBoxOutlineBlankIcon from '@mui/icons-material/CheckBoxOutlineBlank';

function App() {
  const [bet, setBet] = useState(100000);
  const [myGames, setMyGames] = useState([]);
  const [selectedGame, setSelectedGame] = useState(undefined);
  const [updating, setUpdating] = useState(true);

  const { status } = useWallet();

  const connectedWallet = useConnectedWallet();

  useEffect(() => {
    const prefetch = async () => {
      if (connectedWallet) {
        const result = await query.getGames(connectedWallet);
        setMyGames(result.games);
      }
      setUpdating(false)
    }
    prefetch()
  }, [connectedWallet])

  const handleBetChange = (event) => {
    setBet(event.target.value);
  };
  const onClickStartGame = async () => {
    setUpdating(true);
    await execute.createGame(connectedWallet, bet);
    const result = await query.getGames(connectedWallet);
    setMyGames(result.games);
    setUpdating(false);
  }

  const onClickJoinGame = async (gameId) => {
    setUpdating(true);
    await execute.joinGame(connectedWallet, gameId, bet);
    setSelectedGame(await query.getGame(connectedWallet, gameId));
    setUpdating(false);
  }

  const onClickMove = async (x, y) => {
    setUpdating(true);
    await execute.makeMove(connectedWallet, selectedGame.game.game_id, x, y);
    setUpdating(false);
  }

  const onClickLoadGame = async (gameId) => {
    setUpdating(true);
    setSelectedGame(await query.getGame(connectedWallet, gameId));
    setUpdating(false);
  };

  const onClickWithdrawPrice = async (gameId) => {
    setUpdating(true);
    await execute.withdrawPrice(connectedWallet, gameId);
    setUpdating(false);
  };

  const darkTheme = createTheme({
    palette: {
      mode: 'dark',
      primary: {
        main: '#1976d2',
      },
    },
  });

  const Cell = (cell) => {
    switch (cell.value) {
      case 100:
        return <CircleIcon></CircleIcon>;
      case 1:
        return <CloseIcon></CloseIcon>;
    }
    return <CheckBoxOutlineBlankIcon></CheckBoxOutlineBlankIcon>;
  }

  return (
    <div className="App">
      <ThemeProvider theme={darkTheme}>
        <Box sx={{ flexGrow: 1 }}>
          <AppBar position="static">
            <Toolbar>
              <ConnectWallet />
            </Toolbar>
          </AppBar>
        </Box>
      </ThemeProvider>
      <Container>
        <Grid container spacing={2}>
          <Grid item xs={6}>
            <FormControl fullWidth sx={{ m: 1 }} variant="standard">
              <InputLabel htmlFor="standard-adornment-amount">Amount</InputLabel>
              <Input
                id="standard-adornment-amount"
                value={bet}
                onChange={handleBetChange}
                startAdornment={<InputAdornment position="start">$</InputAdornment>}
              />
            </FormControl>
            <Button onClick={onClickStartGame}>Start Game</Button>
            <ul>
              {
                myGames.map(game => <li key={game.game_id}>
                  <b>Game: {game.game_id}</b>
                  <Button onClick={() => onClickLoadGame(game.game_id)}>
                    Load game
                  </Button>
                  <Button onClick={() => onClickJoinGame(game.game_id)}>
                    Join game
                  </Button>
                  <Button onClick={() => onClickWithdrawPrice(game.game_id)}>
                    Withdraw price
                  </Button>
                </li>
                )
              }
            </ul>
          </Grid>
          <Grid item xs={6}>
            <>{(selectedGame) && (selectedGame.game.grid.map((row, x) =>
              <div key={`row-${x}`}>
                {row.map((cell, y) =>
                  <Button disabled={cell !== 0} variant="outlined" size="large" key={`cell-${x}-${y}`} onClick={() => onClickMove(x, y)}>
                    <Cell value={cell}></Cell>
                  </Button>)}
              </div>
            ))}
            {(selectedGame) && (
              <p>Current move: {selectedGame.game.next_player}</p>
            )}            
            </>
          </Grid>
        </Grid>

      </Container>
    </div>
  )
}

export default App
