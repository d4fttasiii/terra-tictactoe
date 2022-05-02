import {
  WalletStatus,
  useWallet
} from '@terra-money/wallet-provider';
import Button from '@mui/material/Button';
import AccountBalanceWalletIcon from '@mui/icons-material/AccountBalanceWallet';

export const ConnectWallet = () => {
  const {
    status,
    connect,
    disconnect,
    wallets,
  } = useWallet()

  function formatWalletAddress(address) {
    return `${address.substring(0, 7)}...${address.substring(address.length - 6)}`;
  }

  return (
    <div>
      {status === WalletStatus.WALLET_NOT_CONNECTED && (
        <Button variant="outlined" onClick={() => connect("EXTENSION")}>
          <AccountBalanceWalletIcon></AccountBalanceWalletIcon>
          Connect
        </Button>
      )}
      {status === WalletStatus.WALLET_CONNECTED && (
        <>
          <Button variant="outlined" onClick={() => disconnect()}>
            <AccountBalanceWalletIcon></AccountBalanceWalletIcon>
            Disconnect {formatWalletAddress(wallets[0].terraAddress)}
          </Button>
        </>
      )}
    </div>
  )
}
