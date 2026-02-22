<script setup lang="ts">
import { computed, reactive, watchEffect } from 'vue'
import { useQuery, useMutation } from '@vue/apollo-composable'
import { 
  GET_LOBBY_QUERY, ROLL_DICE_MUTATION, RESOLVE_FORCED_DEAL_MUTATION, 
  RESOLVE_PURCHASE_MUTATION, RESOLVE_FIRST_CLASS_MUTATION, 
  RESOLVE_AIRPORT_DECISION_MUTATION, RESOLVE_AIRPORT_DESTINATION_MUTATION, 
  RESOLVE_TARGET_SELECTION_MUTATION, ROLL_DUEL_DIE_MUTATION,
  PLACE_BID_MUTATION, RESOLVE_AUCTION_MUTATION,
  RESOLVE_JAIL_DECISION_MUTATION
} from '../../graphql/operations'
import Passport from './Passport.vue'
import Stamp from './Stamp.vue'
import DicePanel from './DicePanel.vue'
import ActivityLog from './ActivityLog.vue'
import CardStack from './CardStack.vue'
import GameToken from './GameToken.vue'
import CardHand from '../CardHand.vue'
import PlayerSelectionModal from './PlayerSelectionModal.vue'
import WinnerAnnouncement from './WinnerAnnouncement.vue'

const props = defineProps<{
  code: string
}>()

// Fetch lobby data to get players and their characters
const { result } = useQuery(GET_LOBBY_QUERY, () => ({
  code: props.code
}), {
  pollInterval: 1000 // Poll every 1 second for faster updates
});

const username = localStorage.getItem('username') || '';

const { mutate: rollDiceMutation } = useMutation(ROLL_DICE_MUTATION);
const { mutate: resolveForcedDealMutation } = useMutation(RESOLVE_FORCED_DEAL_MUTATION);
const { mutate: resolvePurchaseMutation } = useMutation(RESOLVE_PURCHASE_MUTATION);
const { mutate: resolveFirstClassMutation } = useMutation(RESOLVE_FIRST_CLASS_MUTATION);
const { mutate: resolveAirportDecisionMutation } = useMutation(RESOLVE_AIRPORT_DECISION_MUTATION);
const { mutate: resolveAirportDestinationMutation } = useMutation(RESOLVE_AIRPORT_DESTINATION_MUTATION);
const { mutate: resolveTargetSelectionMutation } = useMutation(RESOLVE_TARGET_SELECTION_MUTATION);
const { mutate: rollDuelDieMutation } = useMutation(ROLL_DUEL_DIE_MUTATION);
const { mutate: placeBidMutation } = useMutation(PLACE_BID_MUTATION);
const { mutate: resolveAuctionMutation } = useMutation(RESOLVE_AUCTION_MUTATION);
const { mutate: resolveJailDecisionMutation } = useMutation(RESOLVE_JAIL_DECISION_MUTATION);

interface Property {
  name: string;
  color: string;
  diameter: number;
  column: 'left' | 'right';
  destination_id?: number | null;
}

interface Player {
  character: 'seal' | 'capybara' | 'cat' | 'dog';
  position: number;
  name: string;
  in_jail: boolean;
  consecutive_doubles: number;
  money: number;
  properties: Property[];
  hereAndNowCards: Array<{ id: string; description: string }>;
  chanceCards: Array<{ id: string; description: string }>;
}

interface GameState {
  players: Player[];
  currentTurnIndex: number;
  diceValue1: number;
  diceValue2: number;
  isRolling: boolean;
  forcedDealActive: boolean;
  isMoving: boolean;
  awaitingAction: boolean;
  pendingPurchase: { destId: number; destName: string; price: number; buyerIdx: number } | null;
  pendingFirstClass: { buyerIdx: number } | null;
  pendingAirportDecision: { buyerIdx: number } | null;
  pendingAirportDestination: { buyerIdx: number } | null;
  isGameOver: boolean;
  winnerName: string | null;
  moneyNotifications: Array<{ id: number; amount: string; type: 'plus' | 'minus'; x: number; y: number; playerName: string }>;
  pickingTarget: boolean;
  targetSelection: { action: string; cardId: string | null; selectorIdx: number } | null;
  diceDuel: {
    challengerIdx: number;
    targetIdx: number;
    challengerDie1: number | null;
    challengerDie2: number | null;
    targetDie1: number | null;
    targetDie2: number | null;
  } | null;
  pendingAuction: { destId: number; destName: string; currentBid: number; highestBidderIdx: number | null } | null;
  auctionTimer: number;
  activityLog: Array<{ playerIdx: number | null; message: string }>;
  isJailDecision: boolean;
}

// Game simulation state (local representation of server state)
const gameState = reactive<GameState>({
  players: [],
  currentTurnIndex: 0,
  diceValue1: 1,
  diceValue2: 3,
  isRolling: false,
  forcedDealActive: false,
  isMoving: false,
  awaitingAction: false,
  pendingPurchase: null,
  pendingFirstClass: null,
  pendingAirportDecision: null,
  pendingAirportDestination: null,
  isGameOver: false,
  winnerName: null,
  moneyNotifications: [],
  pickingTarget: false,
  targetSelection: null,
  diceDuel: null,
  pendingAuction: null,
  auctionTimer: 0,
  activityLog: [],
  isJailDecision: false,
})

let auctionInterval: any = null;

// Watch for auction state changes to manage timer
import { watch } from 'vue'
watch(() => gameState.pendingAuction, (newVal, oldVal) => {
  // If no auction, clear timer
  if (!newVal) {
    if (auctionInterval) {
      clearInterval(auctionInterval);
      auctionInterval = null;
    }
    gameState.auctionTimer = 0;
    return;
  }

  // If new auction OR bid changed (currentBid changed), reset timer
  if (!oldVal || newVal?.currentBid !== oldVal?.currentBid) {
    if (auctionInterval) clearInterval(auctionInterval);
    gameState.auctionTimer = 5; // 5 seconds
    
    auctionInterval = setInterval(() => {
      gameState.auctionTimer--;
      if (gameState.auctionTimer <= 0) {
        clearInterval(auctionInterval);
        handleResolveAuction();
      }
    }, 1000);
  }
}, { deep: true });

let notificationId = 0;

// Sync local state with server data
watchEffect(() => {
  const res = result?.value;
  if (res?.getLobby) {
    const lobby = res.getLobby
    
    // 1. Sync Game State
    if (lobby.gameState) {
      gameState.isGameOver = lobby.gameState.isGameOver;
      gameState.winnerName = lobby.gameState.winnerName;

      // Always sync decision states so modals appear even if slightly delayed by animations
      gameState.pendingPurchase = lobby.gameState.pendingPurchase || null
      gameState.pendingFirstClass = lobby.gameState.pendingFirstClass || null
      gameState.pendingAirportDecision = lobby.gameState.pendingAirportDecision || null
      gameState.pendingAirportDestination = lobby.gameState.pendingAirportDestination || null
      gameState.targetSelection = lobby.gameState.targetSelection || null
      if (lobby.gameState.diceDuel) {
        gameState.diceDuel = {
          challengerIdx: lobby.gameState.diceDuel.challengerIdx,
          targetIdx: lobby.gameState.diceDuel.targetIdx,
          challengerDie1: lobby.gameState.diceDuel.challengerDie1,
          challengerDie2: lobby.gameState.diceDuel.challengerDie2,
          targetDie1: lobby.gameState.diceDuel.targetDie1,
          targetDie2: lobby.gameState.diceDuel.targetDie2
        }
      } else {
        gameState.diceDuel = null
      }
      // Check for auction
      // If we already have a pendingAuction and the ID/bid matches, update it (to keep the local object ref if possible, or just replace)
      // Actually, standard replacing is fine, the watch will handle diffing deeply or property-wise
      gameState.pendingAuction = lobby.gameState.pendingAuction || null
      gameState.activityLog = lobby.gameState.activityLog || []

      // Sync turn and dice, BUT only if we are not currently animating a roll ourselves
      if (!gameState.isRolling && !gameState.isMoving) {
        gameState.currentTurnIndex = lobby.gameState.currentTurnIndex
        if (lobby.gameState.lastDie1) gameState.diceValue1 = lobby.gameState.lastDie1
        if (lobby.gameState.lastDie2) gameState.diceValue2 = lobby.gameState.lastDie2
        gameState.awaitingAction = lobby.gameState.awaitingAction
        gameState.isJailDecision = lobby.gameState.isJailDecision || false
      }
      gameState.forcedDealActive = lobby.gameState.isForcedDeal
    }

    // 2. Sync Players
    if (lobby.players) {
      // Map server players to local format
      const newPlayers = lobby.players.map((p: any) => ({
        character: p.character as 'seal' | 'capybara' | 'cat' | 'dog',
        position: p.position || 0,
        name: p.username,
        in_jail: p.inJail || false,
        consecutive_doubles: p.consecutiveDoubles || 0,
        money: p.money ?? 1500,
        properties: (p.properties || []).map((prop: any) => ({
          name: prop.name,
          color: prop.color,
          diameter: prop.diameter,
          column: prop.column,
          destination_id: prop.destinationId
        })),
        hereAndNowCards: p.hereAndNowCards || [],
        chanceCards: p.chanceCards || []
      }))

      // Detect money changes for animations
      newPlayers.forEach((np: any) => {
        const oldP = gameState.players.find(p => p.name === np.name);
        if (oldP && oldP.money !== np.money) {
          const diff = np.money - oldP.money;
          const type: 'plus' | 'minus' = diff > 0 ? 'plus' : 'minus';
          const amountStr = (diff > 0 ? '+' : '-') + 'M' + Math.abs(diff);
          
          // Add notification
          const id = notificationId++;
          gameState.moneyNotifications.push({
            id,
            amount: amountStr,
            type,
            x: 0, 
            y: 0,
            playerName: np.name as string
          });

          // Remove after 2 seconds
          setTimeout(() => {
            gameState.moneyNotifications = gameState.moneyNotifications.filter(n => n.id !== id);
          }, 2000);
        }
      });

      gameState.players = newPlayers;
    }
  }
})

const isMyTurn = computed(() => {
  const currentPlayer = gameState.players[gameState.currentTurnIndex]
  return !!(currentPlayer && currentPlayer.name === username)
})

const isMyJailDecision = computed(() => {
  return gameState.isJailDecision && isMyTurn.value;
})

const myHasJailFreeCard = computed(() => {
  if (!myPlayerData.value) return false;
  const hasChanceJailFree = myPlayerData.value.chanceCards.some(c => c.id === 'jail_free');
  const hasHnNJailFree = myPlayerData.value.hereAndNowCards.some(c => c.id.includes('jail_free')); 
  return hasChanceJailFree || hasHnNJailFree;
})

const canBuy = computed(() => {
  if (!gameState.pendingPurchase) return false
  return gameState.players[gameState.pendingPurchase?.buyerIdx!]?.name === username
})

const canBuyFirstClass = computed(() => {
  if (!gameState.pendingFirstClass) return false
  return gameState.players[gameState.pendingFirstClass?.buyerIdx!]?.name === username
})

const canFly = computed(() => {
  if (!gameState.pendingAirportDecision) return false
  return gameState.players[gameState.pendingAirportDecision?.buyerIdx!]?.name === username
})

const canPickDestination = computed(() => {
  if (!gameState.pendingAirportDestination) return false
  return gameState.players[gameState.pendingAirportDestination?.buyerIdx!]?.name === username
})

const myPlayerData = computed(() => {
  return gameState.players.find(p => p.name === username);
})

const myPlayerIdx = computed(() => {
  return gameState.players.findIndex(p => p.name === username);
})


// Exact stamp colors from Stamp.vue
const COLORS = {
  brown: '#8B4513',
  lightblue: '#79F7EF',
  pink: '#D61A8B',
  orange: '#F0760C',
  red: '#CC0000',
  yellow: '#FBFF00',
  green: '#04910D',
  darkblue: '#0D47A1',
} as const

// Board space types
interface Space {
  type: 'start' | 'destination' | 'chance' | 'airport' | 'here_and_now' | 'first_class' | 'just_visiting' | 'free_parking' | 'go_to_jail'
  name?: string
  price?: number
  tax?: number
  color?: keyof typeof COLORS
  id?: number
}

const boardSpaces: Space[] = [
  { type: 'start' },

  { type: 'destination', name: 'Madrid', price: 60, tax: 40, color: 'brown', id: 22 },
  { type: 'chance' },
  { type: 'destination', name: 'Giethoorn', price: 60, tax: 40, color: 'brown', id: 21 },
  { type: 'airport' },
  { type: 'here_and_now' },
  { type: 'destination', name: 'Taipei', price: 100, tax: 60, color: 'lightblue', id: 20 },
  { type: 'first_class' },
  { type: 'destination', name: 'Cape Town', price: 100, tax: 60, color: 'lightblue', id: 19 },
  { type: 'destination', name: 'Queenstown', price: 100, tax: 60, color: 'lightblue', id: 18 },
  { type: 'just_visiting' },
  { type: 'destination', name: 'Sydney', price: 160, tax: 100, color: 'pink', id: 17 },
  { type: 'chance' },
  { type: 'destination', name: 'Amsterdam', price: 160, tax: 100, color: 'pink', id: 16 },
  { type: 'destination', name: 'New York', price: 160, tax: 100, color: 'pink', id: 15 },
  { type: 'here_and_now' },
  { type: 'destination', name: 'Tokyo', price: 200, tax: 120, color: 'orange', id: 14 },
  { type: 'first_class' },
  { type: 'destination', name: 'Moscow', price: 200, tax: 120, color: 'orange', id: 13 },
  { type: 'destination', name: 'London', price: 200, tax: 120, color: 'orange', id: 12 },
  { type: 'free_parking' },
  { type: 'destination', name: 'Belgrade', price: 260, tax: 140, color: 'red', id: 11 },
  { type: 'chance' },
  { type: 'destination', name: 'Athens', price: 260, tax: 140, color: 'red', id: 10 },
  { type: 'destination', name: 'Belfast', price: 260, tax: 140, color: 'red', id: 9 },
  { type: 'here_and_now' },
  { type: 'destination', name: 'Santiago', price: 300, tax: 180, color: 'yellow', id: 8 },
  { type: 'destination', name: 'Mexico City', price: 300, tax: 180, color: 'yellow', id: 7 },
  { type: 'first_class' },
  { type: 'destination', name: 'Warsaw', price: 300, tax: 180, color: 'yellow', id: 6 },
  { type: 'go_to_jail' },
  { type: 'destination', name: 'Istanbul', price: 360, tax: 200, color: 'green', id: 5 },
  { type: 'destination', name: 'Lisbon', price: 360, tax: 200, color: 'green', id: 4 },
  { type: 'chance' },
  { type: 'destination', name: 'Riga', price: 360, tax: 200, color: 'green', id: 3 },
  { type: 'here_and_now' },
  { type: 'airport' },
  { type: 'destination', name: 'Hong Kong', price: 400, tax: 240, color: 'darkblue', id: 2 },
  { type: 'first_class' },
  { type: 'destination', name: 'Lima', price: 400, tax: 240, color: 'darkblue', id: 1 },
]

// Coordinate mapping for all 40 board positions (0-39)
// Returns {top, left} as percentages relative to the board
// Tokens should be positioned ON the board fields (outer edge), not in center
const getSpaceCoordinates = (position: number): { top: string; left: string } => {
  // Board layout:
  // Corner size is ~11.5% of board
  // Each regular space is ~8.56% of board width
  const cornerPct = 11.5
  const spaceWidth = (100 - 2 * cornerPct) / 9 // ~8.56%
  
  // Token position within a field (centered, near color band)
  const fieldOffset = 6 // offset from edge in percentage
  
  // Position 0: START (bottom-right corner)
  if (position === 0) {
    return { top: `${100 - cornerPct / 2}%`, left: `${100 - cornerPct / 2}%` }
  }
  
  // Positions 1-9: Bottom row (right to left, moving towards JAIL)
  if (position >= 1 && position <= 9) {
    const spaceIndex = position - 1
    const leftPos = 100 - cornerPct - spaceIndex * spaceWidth - spaceWidth / 2
    return {
      top: `${100 - fieldOffset}%`,
      left: `${leftPos}%`
    }
  }
  
  // Position 10: JUST VISITING (bottom-left corner)
  if (position === 10) {
    return { top: `${100 - cornerPct / 2}%`, left: `${cornerPct / 2}%` }
  }
  
  // Positions 11-19: Left column (bottom to top)
  if (position >= 11 && position <= 19) {
    const spaceIndex = position - 11
    const topPos = 100 - cornerPct - spaceIndex * spaceWidth - spaceWidth / 2
    return {
      top: `${topPos}%`,
      left: `${fieldOffset}%`
    }
  }
  
  // Position 20: FREE PARKING (top-left corner)
  if (position === 20) {
    return { top: `${cornerPct / 2}%`, left: `${cornerPct / 2}%` }
  }
  
  // Positions 21-29: Top row (left to right)
  if (position >= 21 && position <= 29) {
    const spaceIndex = position - 21
    const leftPos = cornerPct + spaceIndex * spaceWidth + spaceWidth / 2
    return {
      top: `${fieldOffset}%`,
      left: `${leftPos}%`
    }
  }
  
  // Position 30: GO TO JAIL (top-right corner)
  if (position === 30) {
    return { top: `${cornerPct / 2}%`, left: `${100 - cornerPct / 2}%` }
  }
  
  // Positions 31-39: Right column (top to bottom)
  if (position >= 31 && position <= 39) {
    const spaceIndex = position - 31
    const topPos = cornerPct + spaceIndex * spaceWidth + spaceWidth / 2
    return {
      top: `${topPos}%`,
      left: `${100 - fieldOffset}%`
    }
  }
  
  // Fallback to START
  return { top: `${100 - cornerPct / 2}%`, left: `${100 - cornerPct / 2}%` }
}

// Get rotation for token based on board position (facing forward)
const getSpaceRotation = (position: number): string => {
  // Bottom row (0 -> 9): Moving West (Start is at the beginning of this journey)
  if (position >= 0 && position <= 9) return 'rotate(90deg)'
  // Left column (10 -> 19): Moving North (Change starts AT Jail)
  if (position >= 10 && position <= 19) return 'rotate(180deg)'
  // Top row (20 -> 29): Moving East (Change starts AT Free Parking)
  if (position >= 20 && position <= 29) return 'rotate(270deg)'
  // Right column (30 -> 39): Moving South (Change starts AT Go To Jail)
  if (position >= 30 && position <= 39) return 'rotate(0deg)'
  
  return 'rotate(0deg)'
}

const handleResolveJailDecision = async (action: 'PayFine' | 'UseCard' | 'Roll') => {
  if (!isMyJailDecision.value) return;
  
  try {
     const response = await resolveJailDecisionMutation({
       code: props.code,
       username: username,
       action
     });

     const r = response?.data?.resolveJailDecision;
     if (r) {
        if (r.die1 > 0) {
           // ... (rest of the dice removal/anim logic)
           gameState.diceValue1 = r.die1;
           gameState.diceValue2 = r.die2;
           if (!r.wentToJail) {
               gameState.isRolling = true;
               setTimeout(() => {
                 gameState.isRolling = false;
               }, 600);
           }
        }
        // Always update turn and decision state from response
        gameState.currentTurnIndex = r.currentTurnIndex;
        gameState.isJailDecision = false; // Reset locally
     }
  } catch (err: any) {
    alert(err.message);
  }
}

// Roll dice function
const rollDice = async () => {
  if (!isMyTurn.value || gameState.isRolling || gameState.isMoving || gameState.forcedDealActive) return
  
  gameState.isRolling = true

  try {
    // 1. Start Animation
    // Simulate rolling animation time
    const animPromise = new Promise(resolve => setTimeout(resolve, 600))
    
    // 2. Call Backend
    const response = await rollDiceMutation({
      code: props.code,
      username: username
    })
    
    // Fix: Access data safely
    const result = response?.data?.rollDice
    if (!result) throw new Error("No result from roll")

    await animPromise // Wait for animation time
    
    // 3. Update Dice Values
    gameState.diceValue1 = result.die1
    gameState.diceValue2 = result.die2
    gameState.isRolling = false
    
    // 4. Handle Movement / Forced Deal
    if (result.isForcedDeal && !result.wentToJail) {
      // Delay modal so user sees the handshake icon first
        gameState.forcedDealActive = true
    } else {
      // Small delay to let user see dice values before jumping
      await new Promise(resolve => setTimeout(resolve, 600))
      
      // Move the current player locally (server updated it, but we animate it)
      await movePlayer(result.newPosition)
      
      if (result.wentToJail) {
        // Maybe show jail notification
      }
    }
  } catch (e) {
    console.error("Roll failed:", e)
    gameState.isRolling = false
  }
}

// Move player animation
const movePlayer = async (targetPosition: number) => {
  const player = gameState.players[gameState.currentTurnIndex]
  if (!player) return
  
  gameState.isMoving = true
  
  // Jump directly to the destination
  player.position = targetPosition
  
  // Give a moment for the animation to play out
  await new Promise(resolve => setTimeout(resolve, 400))
  
  gameState.isMoving = false
}

// Handle Forced Deal choices
const handleSneakySwap = async () => {
  // Instead of calling mutation immediately, we switch to target selection mode
  gameState.pickingTarget = true
}

const executeSwap = async (targetUsername: string) => {
  try {
    await resolveForcedDealMutation({
      code: props.code,
      username: username,
      action: 'SneakySwap',
      target: targetUsername
    })
    gameState.forcedDealActive = false
    gameState.pickingTarget = false
  } catch (e) {
    console.error("Swap failed:", e)
  }
}

const handleMoveN = async () => {
  try {
    await resolveForcedDealMutation({
      code: props.code,
      username: username,
      action: 'move'
    })
    gameState.forcedDealActive = false
    // Manually jump to predicted position (current + die2) to feel responsive
    // Poll will confirm it shortly
    const player = gameState.players[gameState.currentTurnIndex]
    if (player) {
       player.position = (player.position + gameState.diceValue2) % 40
    }
  } catch (e) {
    console.error("Move failed:", e)
  }
}

// Handle Purchase Decision
const handlePurchaseDecision = async (buy: boolean) => {
  try {
    await resolvePurchaseMutation({
      code: props.code,
      username: username,
      buy: buy
    })
    gameState.pendingPurchase = null
  } catch (e) {
    console.error("Purchase decision failed:", e)
  }
}

// Handle First Class Decision
const handleFirstClassDecision = async (buy: boolean) => {
  try {
    await resolveFirstClassMutation({
      code: props.code,
      username: username,
      buy: buy
    })
    gameState.pendingFirstClass = null
  } catch (e) {
    console.error("First Class decision failed:", e)
  }
}

// Handle Target Selection (Dice Duel or Stamp Swap)
const handleSelectTarget = async (targetUsername: string) => {
  try {
    await resolveTargetSelectionMutation({
      code: props.code,
      username: username,
      targetUsername: targetUsername
    })
    gameState.targetSelection = null
  } catch (e) {
    console.error("Target selection failed:", e)
  }
}

// Handle Place Bid
const handlePlaceBid = async (amount: number) => {
  try {
    // Optimistic: update locally to feel instant? 
    // No, wait for server to ensure validation (money etc)
    await placeBidMutation({
      code: props.code,
      username: username,
      amount: amount
    })
  } catch (e) {
    console.error("Bid failed:", e)
  }
}

// Resolve Auction (Timer expired)
const handleResolveAuction = async () => {
  if (!gameState.pendingAuction) return;
  try {
    await resolveAuctionMutation({
      code: props.code
    })
  } catch (e) {
    console.error("Resolve auction failed:", e)
  }
}

// Handle Dice Duel Roll
const handleRollDuelDie = async () => {
  try {
    await rollDuelDieMutation({
      code: props.code,
      username: username
    })
    // UI will update via poll
  } catch (e) {
    console.error("Duel roll failed:", e)
  }
}

// Handle Airport Flight Decision
const handleAirportDecision = async (buy: boolean) => {
  try {
    await resolveAirportDecisionMutation({
      code: props.code,
      username: username,
      buyFlight: buy
    })
    gameState.pendingAirportDecision = null
  } catch (e) {
    console.error("Airport decision failed:", e)
  }
}

// Handle Airport Destination Selection
const handleSelectDestination = async (position: number) => {
    if (!gameState.pendingAirportDestination || !canPickDestination.value) return
    
    try {
      await resolveAirportDestinationMutation({
        code: props.code,
        username: username,
        targetPosition: position
      })
      gameState.pendingAirportDestination = null
    
    // Optimistic local move
    const player = gameState.players[gameState.currentTurnIndex]
    if (player) {
      player.position = position
    }
  } catch (e) {
    console.error("Airport destination failed:", e)
  }
}

// Get current player
const currentPlayer = computed(() => gameState.players[gameState.currentTurnIndex])

// Get Auction Top Bidder Name safely
const auctionTopBidder = computed(() => {
  if (!gameState.pendingAuction || gameState.pendingAuction?.highestBidderIdx === null || gameState.pendingAuction?.highestBidderIdx === undefined) return 'No bids yet'
  const player = gameState.players[gameState.pendingAuction?.highestBidderIdx!]
  return player ? player.name : 'Unknown'
})

// Separate spaces by position on the board
const bottomRow = computed(() => boardSpaces.slice(1, 10).reverse())
const leftColumn = computed(() => boardSpaces.slice(11, 20))
const topRow = computed(() => boardSpaces.slice(21, 30))
const rightColumn = computed(() => boardSpaces.slice(31, 40))

function getColorStyle(color?: string): string {
  if (!color) return 'transparent'
  if (color in COLORS) {
    return COLORS[color as keyof typeof COLORS]
  }
  if (color === 'gray' || color === 'grey') return '#808080'
  return color // Assume it might be a hex or actual color name
}

const getSpaceIcon = (type: string): string => {
  switch (type) {
    case 'chance': return '?'
    case 'airport': return '✈'
    case 'here_and_now': return '⭐'
    case 'first_class': return '💎'
    default: return ''
  }
}

// Check if a property is owned by anyone
const isPropertyOwned = (destId: number) => {
  return gameState.players.some(p => p.properties.some((prop: any) => prop.destination_id === destId));
}

// Logic for space selection (Airport flight)
const getSpacePosition = (type: 'bottom' | 'left' | 'top' | 'right' | 'corner', index: number) => {
  if (type === 'corner') return index; // 0, 10, 20, 30
  if (type === 'bottom') return 9 - index; // bottomRow is reverse() of 1-9, so index 0 is pos 9
  if (type === 'left') return 11 + index;
  if (type === 'top') return 21 + index;
  if (type === 'right') return 31 + index;
  return 0;
}

const handleSpaceClick = (type: 'bottom' | 'left' | 'top' | 'right' | 'corner', index: number) => {
  const pos = getSpacePosition(type, index);
  const space = boardSpaces[pos];
  
  if (space && gameState.pendingAirportDestination && canPickDestination.value) {
    if (space.type === 'destination' || space.type === 'first_class') {
      handleSelectDestination(pos);
    }
  }
}

const isSpaceSelectable = (space: Space | undefined) => {
  if (!space) return false;
  return gameState.pendingAirportDestination && canPickDestination.value && (space.type === 'destination' || space.type === 'first_class');
}

// Zone mapping: 
// zone-bottom-right -> player index 0
// zone-bottom-left  -> player index 1
// zone-top-left     -> player index 2
// zone-top-right    -> player index 3
const getPlayerByZone = (zone: 'bottom-right' | 'bottom-left' | 'top-left' | 'top-right') => {
  const indexMap = {
    'bottom-right': 0,
    'bottom-left': 1,
    'top-left': 2,
    'top-right': 3
  };
  return gameState.players[indexMap[zone]];
}
</script>

<template>
  <div class="game-layout">
  <!-- Right Side Player Info Panel (Now Left Side) -->
  <div class="economy-panel">
      <div class="panel-header">
        <h2>💰 World Bank</h2>
      </div>
      
      <!-- Global Money List -->
      <div class="global-money-list">
        <div 
          v-for="(player, idx) in gameState.players" 
          :key="'money-'+idx"
          class="mini-player-card"
          :class="{ 'me-gold': player.name === username }"
        >
          <div class="mini-info">
            <div class="mini-token-box">
              <GameToken :type="player.character" />
            </div>
            <span class="mini-name">{{ player.name }}</span>
          </div>
          <div class="mini-money-wrap">
             <span class="mini-money">M{{ player.money }}</span>
             <!-- Small floating animations within the card context -->
             <TransitionGroup name="money-anim">
               <span 
                 v-for="note in gameState.moneyNotifications.filter(n => n.playerName === player.name)" 
                 :key="note.id" 
                 class="float-money"
                 :class="note.type"
               >
                 {{ note.amount }}
               </span>
             </TransitionGroup>
          </div>
        </div>
      </div>

      <div class="portfolio-divider">
        <h3>🏠 My Properties</h3>
      </div>

      <div class="my-portfolio">
        <template v-for="player in gameState.players" :key="'props-'+player.name">
          <div v-if="player.name === username" class="personal-props-area">
            <!-- Mini Passport View -->
            <div class="mini-passport-container">
              <Passport 
                :properties="player.properties"
              />
            </div>

            <div v-if="!player.properties || player.properties.length === 0" class="no-props-box">
              You don't own any properties yet.
            </div>
            <div class="props-grid">
              <div 
                v-for="(prop, pIdx) in player.properties" 
                :key="pIdx"
                class="prop-pill"
              >
                  <span class="prop-dot" :style="{ background: getColorStyle(prop.color) }"></span>
                <span class="prop-label">{{ prop.name }}</span>
              </div>
            </div>
          </div>
        </template>
      </div>

  </div>

  <div class="board-container">
    <div class="board">
      <!-- Top row -->
      <div class="board-row top-row">
        <!-- FREE PARKING corner -->
        <div class="corner-space" :class="{ 'selectable-destination': isSpaceSelectable(boardSpaces[20]) }" @click="handleSpaceClick('corner', 20)">
          <div class="corner-content free-parking">
            <div class="corner-icon">🅿️</div>
            <span class="corner-label">FREE<br>PARKING</span>
          </div>
        </div>
        
        <div class="spaces-row">
          <div 
            v-for="(space, index) in topRow" 
            :key="'top-' + index"
            class="space"
            :class="[space.type, { 'selectable-destination': isSpaceSelectable(space) }]"
            @click="handleSpaceClick('top', index)"
          >
            <div 
              v-if="space.color" 
              class="color-band"
              :style="{ background: getColorStyle(space.color) }"
            ></div>
            <div class="space-content">
              <template v-if="space.type === 'destination'">
                <span class="space-name">{{ space.name }}</span>
                <span class="space-price">M{{ space.price }}</span>
              </template>
              <template v-else>
                <div class="special-content-wrapper">
                  <!-- First Class -->
                  <div v-if="space.type === 'first_class'" class="first-class-content">
                    <span class="special-label">FIRST CLASS</span>
                    <div class="elegant-star-ring-v2">
                      <div class="elegant-star-v2">★</div>
                    </div>
                    <span class="special-price">M100</span>
                  </div>
                  
                  <!-- Airport -->
                  <div v-else-if="space.type === 'airport'" class="airport-content">
                    <span class="special-label">AIRPORT</span>
                    <div class="airport-emoji">✈️</div>
                    <span class="special-price">M100</span>
                  </div>

                  <!-- Here & Now (Preserved) -->
                  <div v-else-if="space.type === 'here_and_now'" class="here-now-content">
                    <div class="globe-motif">
                      <div class="globe-icon-outline">🌎</div>
                    </div>
                    <div class="here-now-text">
                      <span class="text-here">HERE</span>
                      <span class="text-and">&</span>
                      <span class="text-now">NOW</span>
                    </div>
                  </div>

                  <!-- Chance (Preserved) -->
                  <div v-else-if="space.type === 'chance'" class="chance-content">
                    <span class="large-question-mark">?</span>
                    <span class="chance-label">CHANCE</span>
                  </div>

                  <!-- Other special spaces -->
                  <template v-else>
                    <span class="space-icon">{{ getSpaceIcon(space.type) }}</span>
                    <span class="space-type-label">{{ space.type.replace(/_/g, ' ').toUpperCase() }}</span>
                  </template>
                </div>
              </template>
            </div>
            
            <!-- Property Stamp -->
            <div v-if="space.color && !isPropertyOwned(space.id!)" class="property-stamp">
              <Stamp 
                :color-type="space.color === 'darkblue' ? 'blue' : space.color"
                :number="space.id!"
                :label="space.name"
              />
            </div>

          </div>
        </div>
        
        <!-- GO TO JAIL corner -->
        <div class="corner-space" :class="{ 'selectable-destination': isSpaceSelectable(boardSpaces[30]) }" @click="handleSpaceClick('corner', 30)">
          <div class="corner-content go-to-jail">
            <div class="corner-icon">🚔</div>
            <span class="corner-label">GO TO<br>JAIL</span>
          </div>
        </div>
      </div>
      
      <!-- Middle section -->
      <div class="board-middle">
        <!-- Left column -->
        <div class="spaces-column left-column">
          <div 
            v-for="(space, index) in leftColumn" 
            :key="'left-' + index"
            class="space horizontal-space left-side"
            :class="[space.type, { 'selectable-destination': isSpaceSelectable(space) }]"
            @click="handleSpaceClick('left', index)"
          >
            <div 
              v-if="space.color" 
              class="color-band"
              :style="{ background: getColorStyle(space.color) }"
            ></div>
            <div class="space-content">
              <template v-if="space.type === 'destination'">
                <span class="space-name">{{ space.name }}</span>
                <span class="space-price">M{{ space.price }}</span>
              </template>
              <template v-else>
                <div class="special-content-wrapper">
                  <!-- First Class -->
                  <div v-if="space.type === 'first_class'" class="first-class-content">
                    <span class="special-label">FIRST CLASS</span>
                    <div class="elegant-star-ring-v2">
                      <div class="elegant-star-v2">★</div>
                    </div>
                    <span class="special-price">M100</span>
                  </div>
                  
                  <!-- Airport -->
                  <div v-else-if="space.type === 'airport'" class="airport-content">
                    <span class="special-label">AIRPORT</span>
                    <div class="airport-emoji">✈️</div>
                    <span class="special-price">M100</span>
                  </div>

                  <!-- Here & Now (Preserved) -->
                  <div v-else-if="space.type === 'here_and_now'" class="here-now-content">
                    <div class="globe-motif">
                      <div class="globe-icon-outline">🌎</div>
                    </div>
                    <div class="here-now-text">
                      <span class="text-here">HERE</span>
                      <span class="text-and">&</span>
                      <span class="text-now">NOW</span>
                    </div>
                  </div>

                  <!-- Chance (Preserved) -->
                  <div v-else-if="space.type === 'chance'" class="chance-content">
                    <span class="large-question-mark">?</span>
                    <span class="chance-label">CHANCE</span>
                  </div>

                  <!-- Other special spaces -->
                  <template v-else>
                    <span class="space-icon">{{ getSpaceIcon(space.type) }}</span>
                    <span class="space-type-label">{{ space.type.replace(/_/g, ' ').toUpperCase() }}</span>
                  </template>
                </div>
              </template>
            </div>
            <!-- Property Stamp -->
            <div v-if="space.color && !isPropertyOwned(space.id!)" class="property-stamp">
              <Stamp 
                :color-type="space.color === 'darkblue' ? 'blue' : space.color"
                :number="space.id!"
                :label="space.name"
              />
            </div>

          </div>
        </div>
        
        <!-- Center area with 4 passport zones in corners -->
        <div class="board-center">
          <!-- Passport zone TOP-LEFT -->
          <div class="passport-zone zone-top-left">
            <div class="passport-area">
              <Passport 
                v-if="getPlayerByZone('top-left')"
                :properties="getPlayerByZone('top-left')?.properties"
                :player-name="getPlayerByZone('top-left')?.name"
                :character="getPlayerByZone('top-left')?.character"
              />
            </div>
          </div>
          
          <!-- Passport zone TOP-RIGHT -->
          <div class="passport-zone zone-top-right">
            <div class="passport-area">
              <Passport 
                v-if="getPlayerByZone('top-right')"
                :properties="getPlayerByZone('top-right')?.properties"
                :player-name="getPlayerByZone('top-right')?.name"
                :character="getPlayerByZone('top-right')?.character"
              />
            </div>
          </div>
          
          <!-- Deck Placeholders -->
          <div class="card-deck chance-deck">
            <CardStack type="chance" />
          </div>
          <div class="card-deck here-now-deck">
            <CardStack type="here_and_now" />
          </div>

          <!-- Dice in the center with Roll Button -->
          <div class="dice-container-center">
            <!-- Game Over Choice -->
            <template v-if="gameState.isGameOver">
              <WinnerAnnouncement 
                :winnerName="gameState.winnerName"
                :character="gameState.players.find(p => p.name === gameState.winnerName)?.character || 'cat'"
                :properties="gameState.players.find(p => p.name === gameState.winnerName)?.properties || []"
                @backToLobby="$router.push('/lobby')"
              />
            </template>

            <!-- Active Game UI -->
            <template v-else>
              <!-- Jail Decision Panel -->
              <div v-if="gameState.isJailDecision && isMyTurn && !gameState.isGameOver" class="dice-control-panel jail-panel">
                 <div class="jail-header">
                    <h3>🔒 IN JAIL</h3>
                    <p v-if="myPlayerData">Escape options:</p>
                 </div>
                 
                 <div class="jail-options">
                    <button 
                      class="modal-btn jail-opt pay" 
                      @click="handleResolveJailDecision('PayFine')"
                      :disabled="!isMyJailDecision || (myPlayerData?.money || 0) < 100"
                    >
                      <span class="opt-label">PAY M100</span>
                      <span class="opt-desc">Get out now</span>
                    </button>

                    <button 
                      class="modal-btn jail-opt card" 
                      @click="handleResolveJailDecision('UseCard')"
                      :disabled="!isMyJailDecision || !myHasJailFreeCard"
                    >
                      <span class="opt-label">USE CARD</span>
                      <span class="opt-desc">Jail Free</span>
                    </button>

                    <button 
                      class="modal-btn jail-opt roll" 
                      @click="handleResolveJailDecision('Roll')"
                      :disabled="!isMyJailDecision"
                    >
                      <span class="opt-label">ROLL</span>
                      <span class="opt-desc">Try for doubles</span>
                    </button>
                 </div>
              </div>

              <!-- Forced Deal Panel (Compact) -->
              <div v-else-if="gameState.forcedDealActive && isMyTurn" class="dice-control-panel forced-deal-panel">
                 <template v-if="!gameState.pickingTarget">
                    <h3>⚡ Forced Deal!</h3>
                    <p>Choose your action:</p>
                    <div class="modal-buttons vertical">
                      <button 
                        class="modal-btn sneaky" 
                        @click="handleSneakySwap"
                        :disabled="!myPlayerData || myPlayerData.properties.length === 0"
                      >
                        🤝 SNEAKY SWAP
                      </button>
                      <button class="modal-btn move" @click="handleMoveN">
                        🚀 MOVE {{ gameState.diceValue2 }} SPACES
                      </button>
                    </div>
                 </template>
                 <template v-else>
                    <h3>🤝 Who to swap?</h3>
                    <div class="target-selection-grid">
                      <button 
                        v-for="p in gameState.players.filter(p => p.name !== username)" 
                        :key="p.name"
                        class="target-btn"
                        @click="executeSwap(p.name)"
                        :disabled="p.properties.length === 0"
                      >
                        <div class="target-token">
                          <GameToken :type="p.character" />
                        </div>
                        <span class="target-name">{{ p.name }}</span>
                      </button>
                    </div>
                    <button class="modal-btn skip mini" @click="gameState.pickingTarget = false">
                      ⬅ Back
                    </button>
                 </template>
              </div>

              <!-- Dice Panel (Normal or Duel) -->
              <DicePanel 
                v-else
                :diceValue1="gameState.diceValue1" 
                :diceValue2="gameState.diceValue2" 
                :isRolling="gameState.isRolling"
                :isMoving="gameState.isMoving"
                :forcedDealActive="gameState.forcedDealActive"
                :isMyTurn="isMyTurn"
                :currentPlayerName="currentPlayer?.name"
                :players="gameState.players"
                :username="username || ''"
                :isDuel="!!gameState.diceDuel"
                :duelData="gameState.diceDuel ?? undefined"
                @roll="rollDice"
                @rollDuel="handleRollDuelDie"
              />

              <!-- Activity Log -->
              <ActivityLog
                class="activity-log-pos"
                :entries="gameState.activityLog"
                :players="gameState.players.map(p => ({ name: p.name, character: p.character }))"
              />
            </template>
          </div>


          <!-- Purchase Decision Modal -->
          <div v-if="gameState.pendingPurchase && canBuy" class="purchase-modal">
            <div class="modal-content">
              <h3>🏠 Buy Property?</h3>
              <p class="property-name">{{ gameState.pendingPurchase.destName }}</p>
              <p class="property-price">Price: M{{ gameState.pendingPurchase.price }}</p>
              <div class="modal-buttons">
                <button class="modal-btn buy" @click="handlePurchaseDecision(true)">
                  ✅ Buy
                </button>
                <button class="modal-btn skip" @click="handlePurchaseDecision(false)">
                  ❌ Skip
                </button>
              </div>
            </div>
          </div>

          <!-- Auction Modal -->
          <div v-if="gameState.pendingAuction && !gameState.isGameOver" class="auction-modal">
            <div class="modal-content auction">
              <div class="auction-header">
                <span class="auction-icon">🔨</span>
                <h3>AUCTION!</h3>
              </div>
              <p class="property-name">{{ gameState.pendingAuction.destName }}</p>
              
              <div class="auction-stats">
                <div class="stat-box">
                  <span class="stat-label">Current Bid</span>
                  <span class="stat-value bid-amount">M{{ gameState.pendingAuction.currentBid }}</span>
                </div>
                <div class="stat-box">
                  <span class="stat-label">Top Bidder</span>
                  <span class="stat-value bidder-name">
                    {{ auctionTopBidder }}
                  </span>
                </div>
              </div>

              <div class="auction-timer-container">
                <div class="auction-timer-bar">
                  <div class="timer-fill" :style="{ width: (gameState.auctionTimer / 5 * 100) + '%' }"></div>
                </div>
                <span class="timer-text">{{ gameState.auctionTimer }}s</span>
              </div>

              <div class="modal-buttons auction-buttons">
                <button 
                  class="modal-btn bid" 
                  @click="gameState.pendingAuction && handlePlaceBid(gameState.pendingAuction?.currentBid + 20)"
                  :disabled="!myPlayerData || !gameState.pendingAuction || myPlayerData.money < (gameState.pendingAuction?.currentBid || 0) + 20 || gameState.pendingAuction?.highestBidderIdx === myPlayerIdx"
                >
                  +20
                </button>
                <button 
                  class="modal-btn bid" 
                  @click="gameState.pendingAuction && handlePlaceBid(gameState.pendingAuction?.currentBid + 50)"
                  :disabled="!myPlayerData || !gameState.pendingAuction || myPlayerData.money < (gameState.pendingAuction?.currentBid || 0) + 50 || gameState.pendingAuction?.highestBidderIdx === myPlayerIdx"
                >
                  +50
                </button>
                <button 
                  class="modal-btn bid" 
                  @click="gameState.pendingAuction && handlePlaceBid(gameState.pendingAuction?.currentBid + 100)"
                  :disabled="!myPlayerData || !gameState.pendingAuction || myPlayerData.money < (gameState.pendingAuction?.currentBid || 0) + 100 || gameState.pendingAuction?.highestBidderIdx === myPlayerIdx"
                >
                  +100
                </button>
              </div>
            </div>
          </div>

          <!-- First Class Decision Modal -->
          <div v-if="gameState.pendingFirstClass && canBuyFirstClass && !gameState.isGameOver" class="first-class-modal">
            <div class="modal-content">
              <h3>✈️ First Class Stamp</h3>
              <p>Buy a First Class stamp for M100?</p>
              <div class="modal-buttons">
                <button class="modal-btn buy" @click="handleFirstClassDecision(true)">
                  ✅ Buy
                </button>
                <button class="modal-btn skip" @click="handleFirstClassDecision(false)">
                  ❌ Skip
                </button>
              </div>
            </div>
          </div>

          <!-- Airport Decision Modal -->
          <div v-if="gameState.pendingAirportDecision && canFly && !gameState.isGameOver" class="modal-overlay">
            <div class="airport-modal">
              <div class="modal-content airport">
                <div class="modal-icon mini">✈️</div>
                <h3>Take a Flight?</h3>
                <p>Pay M100 to fly to any destination or First Class space!</p>
                <div class="modal-buttons">
                  <button class="modal-btn buy" @click="handleAirportDecision(true)">
                    ✅ Fly
                  </button>
                  <button class="modal-btn skip" @click="handleAirportDecision(false)">
                    ❌ Stay
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- Purchase Decision Modal -->
          <div v-if="gameState.pendingPurchase && canBuy && !gameState.isGameOver" class="purchase-modal" style="z-index: 1000;">
            <div class="modal-content">
              <h3>🏠 Buy Property?</h3>
              <p class="property-name">{{ gameState.pendingPurchase.destName }}</p>
              <p class="property-price">Price: M{{ gameState.pendingPurchase.price }}</p>
              <div class="modal-buttons">
                <button class="modal-btn buy" @click="handlePurchaseDecision(true)">
                  ✅ Buy
                </button>
                <button class="modal-btn skip" @click="handlePurchaseDecision(false)">
                  ❌ Skip
                </button>
              </div>
            </div>
          </div>

          
          <!-- Passport zone BOTTOM-LEFT -->
          <div class="passport-zone zone-bottom-left">
            <div class="passport-area">
              <Passport 
                v-if="getPlayerByZone('bottom-left')"
                :properties="getPlayerByZone('bottom-left')?.properties"
                :player-name="getPlayerByZone('bottom-left')?.name"
                :character="getPlayerByZone('bottom-left')?.character"
              />
            </div>
          </div>
          
          <!-- Passport zone BOTTOM-RIGHT -->
          <div class="passport-zone zone-bottom-right">
            <div class="passport-area">
              <Passport 
                v-if="getPlayerByZone('bottom-right')"
                :properties="getPlayerByZone('bottom-right')?.properties"
                :player-name="getPlayerByZone('bottom-right')?.name"
                :character="getPlayerByZone('bottom-right')?.character"
              />
            </div>
          </div>
        </div>
        
        <!-- Right column -->
        <div class="spaces-column right-column">
          <div 
            v-for="(space, index) in rightColumn" 
            :key="'right-' + index"
            class="space horizontal-space right-side"
            :class="[space.type, { 'selectable-destination': isSpaceSelectable(space) }]"
            @click="handleSpaceClick('right', index)"
          >
            <div 
              v-if="space.color" 
              class="color-band"
              :style="{ background: getColorStyle(space.color) }"
            ></div>
            <div class="space-content">
              <template v-if="space.type === 'destination'">
                <span class="space-name">{{ space.name }}</span>
                <span class="space-price">M{{ space.price }}</span>
              </template>
              <template v-else>
                <div class="special-content-wrapper">
                  <!-- First Class -->
                  <div v-if="space.type === 'first_class'" class="first-class-content">
                    <span class="special-label">FIRST CLASS</span>
                    <div class="elegant-star-ring-v2">
                      <div class="elegant-star-v2">★</div>
                    </div>
                    <span class="special-price">M100</span>
                  </div>
                  
                  <!-- Airport -->
                  <div v-else-if="space.type === 'airport'" class="airport-content">
                    <span class="special-label">AIRPORT</span>
                    <div class="airport-emoji">✈️</div>
                    <span class="special-price">M100</span>
                  </div>

                  <!-- Here & Now (Preserved) -->
                  <div v-else-if="space.type === 'here_and_now'" class="here-now-content">
                    <div class="globe-motif">
                      <div class="globe-icon-outline">🌎</div>
                    </div>
                    <div class="here-now-text">
                      <span class="text-here">HERE</span>
                      <span class="text-and">&</span>
                      <span class="text-now">NOW</span>
                    </div>
                  </div>

                  <!-- Chance (Preserved) -->
                  <div v-else-if="space.type === 'chance'" class="chance-content">
                    <span class="large-question-mark">?</span>
                    <span class="chance-label">CHANCE</span>
                  </div>

                  <!-- Other special spaces -->
                  <template v-else>
                    <span class="space-icon">{{ getSpaceIcon(space.type) }}</span>
                    <span class="space-type-label">{{ space.type.replace(/_/g, ' ').toUpperCase() }}</span>
                  </template>
                </div>
              </template>
            </div>
            <!-- Property Stamp -->
            <div v-if="space.color && !isPropertyOwned(space.id!)" class="property-stamp">
              <Stamp 
                :color-type="space.color === 'darkblue' ? 'blue' : space.color"
                :number="space.id!"
                :label="space.name"
              />
            </div>

          </div>
        </div>
      </div>
      
      <!-- Bottom row -->
      <div class="board-row bottom-row">
        <!-- JUST VISITING corner -->
        <div class="corner-space" :class="{ 'selectable-destination': isSpaceSelectable(boardSpaces[10]) }" @click="handleSpaceClick('corner', 10)">
          <div class="corner-content just-visiting">
            <div class="jail-box">
              <span>JAIL</span>
            </div>
            <span class="corner-label visiting-label">JUST<br>VISITING</span>
          </div>
        </div>
        
        <div class="spaces-row">
          <div 
            v-for="(space, index) in bottomRow" 
            :key="'bottom-' + index"
            class="space bottom-space"
            :class="[space.type, { 'selectable-destination': isSpaceSelectable(space) }]"
            @click="handleSpaceClick('bottom', index)"
          >
            <div 
              v-if="space.color" 
              class="color-band"
              :style="{ background: getColorStyle(space.color) }"
            ></div>
            <div class="space-content">
              <template v-if="space.type === 'destination'">
                <span class="space-name">{{ space.name }}</span>
                <span class="space-price">M{{ space.price }}</span>
              </template>
              <template v-else>
                <div class="special-content-wrapper">
                  <!-- First Class -->
                  <div v-if="space.type === 'first_class'" class="first-class-content">
                    <span class="special-label">FIRST CLASS</span>
                    <div class="elegant-star-ring-v2">
                      <div class="elegant-star-v2">★</div>
                    </div>
                    <span class="special-price">M100</span>
                  </div>
                  
                  <!-- Airport -->
                  <div v-else-if="space.type === 'airport'" class="airport-content">
                    <span class="special-label">AIRPORT</span>
                    <div class="airport-emoji">✈️</div>
                    <span class="special-price">M100</span>
                  </div>

                  <!-- Here & Now (Preserved) -->
                  <div v-else-if="space.type === 'here_and_now'" class="here-now-content">
                    <div class="globe-motif">
                      <div class="globe-icon-outline">🌎</div>
                    </div>
                    <div class="here-now-text">
                      <span class="text-here">HERE</span>
                      <span class="text-and">&</span>
                      <span class="text-now">NOW</span>
                    </div>
                  </div>

                  <!-- Chance (Preserved) -->
                  <div v-else-if="space.type === 'chance'" class="chance-content">
                    <span class="large-question-mark">?</span>
                    <span class="chance-label">CHANCE</span>
                  </div>

                  <!-- Other special spaces -->
                  <template v-else>
                    <span class="space-icon">{{ getSpaceIcon(space.type) }}</span>
                    <span class="space-type-label">{{ space.type.replace(/_/g, ' ').toUpperCase() }}</span>
                  </template>
                </div>
              </template>
            </div>
            <!-- Property Stamp -->
            <div v-if="space.color && !isPropertyOwned(space.id!)" class="property-stamp">
              <Stamp 
                :color-type="space.color === 'darkblue' ? 'blue' : space.color"
                :number="space.id!"
                :label="space.name"
              />
            </div>

          </div>
        </div>
        
        <!-- START corner -->
        <div class="corner-space" :class="{ 'selectable-destination': isSpaceSelectable(boardSpaces[0]) }" @click="handleSpaceClick('corner', 0)">
          <div class="corner-content start">
            <div class="start-diamond-final">
              <div class="start-arrow-final">➜</div>
            </div>
            <span class="corner-label start-label">START</span>
            <span class="start-bonus">Collect M200</span>
          </div>
        </div>
      </div>

      <!-- Dynamic Token Positioning (on the board fields) -->
      <div 
        v-for="(player, idx) in gameState.players" 
        :key="player.character"
        class="positioned-token"
        :style="{
          top: getSpaceCoordinates(player.position).top,
          left: getSpaceCoordinates(player.position).left,
          transform: `translate(-50%, -50%) ${getSpaceRotation(player.position)}`,
          zIndex: 50 + idx
        }"
      >
        <GameToken 
          :type="player.character" 
          :highlight="gameState.currentTurnIndex === idx && idx === myPlayerIdx"
        />
      </div>
    </div>
  </div>


  <!-- Right Side Cards Panel -->
  <div class="cards-panel">
      <div class="panel-header">
        <h2>🃏 My Cards</h2>
      </div>
      
      <CardHand 
        v-if="myPlayerData"
        :code="code"
        :username="username"
        :here-and-now-cards="myPlayerData.hereAndNowCards"
        :chance-cards="myPlayerData.chanceCards"
        :is-my-turn="isMyTurn"
        :in-jail="myPlayerData.in_jail"
        :property-count="myPlayerData.properties.length"
        :players="gameState.players"
      />
  </div>

  <!-- Selection Modal (Overlay) -->
  <PlayerSelectionModal
    v-if="gameState.targetSelection && gameState.targetSelection?.selectorIdx === gameState.players.findIndex(p => p.name === username)"
    :players="gameState.players"
    :selectorIdx="gameState.targetSelection?.selectorIdx!"
    :action="gameState.targetSelection?.action!"
    :username="username"
    @select="handleSelectTarget"
  />
</div>
</template>

<style scoped>
@import url('https://fonts.googleapis.com/css2?family=Oswald:wght@400;500;600;700&family=Roboto:wght@400;500;700&display=swap');

.game-layout {
  display: flex;
  flex-direction: row;
  min-height: 100vh;
  background: #0a1628;
  width: 100%;
}

.board-container {
  flex: 1;
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 5px 15px 40px 15px;
  box-sizing: border-box;
}

.economy-panel {
  width: 280px;
  background: #0c1828; /* Solid background, no blur filter */
  border-right: 1px solid rgba(255, 215, 0, 0.3);
  display: flex;
  flex-direction: column;
  padding: 12px 10px;
  height: 100vh;
  box-sizing: border-box;
  overflow-y: auto;
  box-shadow: 5px 0 20px rgba(0, 0, 0, 0.3);
}

.cards-panel {
  width: 220px;
  background: rgba(12, 24, 40, 0.98);
  border-left: 1px solid rgba(255, 255, 255, 0.1);
  display: flex;
  flex-direction: column;
  padding: 12px 10px;
  height: 100vh;
  box-sizing: border-box;
  overflow-y: auto;
  box-shadow: -5px 0 20px rgba(0, 0, 0, 0.3);
}

.panel-header h2 {
  font-family: 'Oswald', sans-serif;
  color: #FFD700;
  text-align: center;
  margin-bottom: 8px;
  letter-spacing: 2px;
  text-transform: uppercase;
  font-size: 1.1rem;
}

.global-money-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 16px;
}

.mini-player-card {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 6px;
  padding: 4px 8px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border: 1px solid rgba(255, 255, 255, 0.1);
  transition: all 0.3s ease;
  position: relative;
}

.mini-player-card.me-gold {
  border-color: rgba(255, 215, 0, 0.6);
  background: rgba(255, 215, 0, 0.08);
  box-shadow: inset 0 0 15px rgba(255, 215, 0, 0.05), 0 0 10px rgba(255, 215, 0, 0.1);
}

.mini-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.mini-token-box {
  width: 32px;
  height: 32px;
  flex-shrink: 0;
}

.mini-name {
  font-family: 'Oswald', sans-serif;
  font-size: 14px;
  color: white;
  letter-spacing: 0.5px;
}

.mini-money-wrap {
  position: relative;
  display: flex;
  align-items: center;
}

.mini-money {
  font-family: 'Roboto', sans-serif;
  font-size: 16px;
  color: #FFD700;
  font-weight: 700;
}

/* Portfolio Section */
.portfolio-divider {
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  padding-top: 20px;
  margin-bottom: 16px;
}

.portfolio-divider h3 {
  font-family: 'Oswald', sans-serif;
  color: #fbbf24;
  font-size: 0.8rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  margin: 0;
}

.personal-props-area {
  margin-bottom: 10px;
  width: 100%;
}

.mini-passport-container {
  display: flex;
  justify-content: center;
  margin: 20px 0 10px 0; 
  transform: scale(0.75); 
  transform-origin: top center;
  height: 240px; /* Scaled height of body only (320 * 0.75) */
}

.no-props-box {
  color: rgba(255, 255, 255, 0.4);
  font-style: italic;
  font-size: 13px;
  text-align: center;
  padding: 20px;
  background: rgba(255, 255, 255, 0.02);
  border-radius: 12px;
}

.props-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
  gap: 8px;
}

.prop-pill {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 8px;
  padding: 6px 10px;
  display: flex;
  align-items: center;
  gap: 8px;
  border: 1px solid rgba(255, 255, 255, 0.05);
  white-space: nowrap;
  overflow: hidden;
}

.prop-label {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.9);
  overflow: hidden;
  text-overflow: ellipsis;
}

/* Money Animations */
.float-money {
  position: absolute;
  right: 0;
  top: -20px;
  font-weight: 800;
  font-size: 18px;
  pointer-events: none;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.5);
  z-index: 10;
}

.float-money.plus {
  color: #4CAF50;
}

.float-money.minus {
  color: #f44336;
}

.money-anim-enter-active {
  transition: all 0.5s ease-out;
}
.money-anim-leave-active {
  transition: all 1.5s ease-in;
}

.money-anim-enter-from {
  opacity: 0;
  transform: translateY(20px);
}

.money-anim-enter-to {
  opacity: 1;
  transform: translateY(0);
}

.money-anim-leave-to {
  opacity: 0;
  transform: translateY(-40px);
}



/* Custom Scrollbar for props list */
.props-list.scrollable::-webkit-scrollbar {
  width: 4px;
}
.props-list.scrollable::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.05);
}
.props-list.scrollable::-webkit-scrollbar-thumb {
  background: rgba(255, 215, 0, 0.3);
  border-radius: 4px;
}

/* Sidebar styles */
.prop-item {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.9);
  padding: 4px 8px;
  background: rgba(255, 255, 255, 0.03);
  border-radius: 4px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.prop-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.no-props {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.4);
  font-style: italic;
}

/* Sidebar Animations */
.sidebar-anim-enter-active,
.sidebar-anim-leave-active {
  transition: all 0.3s ease;
}

.sidebar-anim-enter-from,
.sidebar-anim-leave-to {
  transform: translateX(100%);
  opacity: 0;
}

.board {
  --board-size: min(90vw, 88vh, 750px);
  --corner-size: calc(var(--board-size) * 0.115);
  --space-width: calc((var(--board-size) - 2 * var(--corner-size)) / 9);
  --space-height: var(--corner-size);
  
  width: var(--board-size);
  height: var(--board-size);
  position: relative;
  /* Blue gradient background like the physical board */
  background: 
    radial-gradient(ellipse at 30% 30%, rgba(38, 89, 255, 0.15) 0%, transparent 50%),
    radial-gradient(ellipse at 70% 70%, rgba(38, 89, 255, 0.1) 0%, transparent 40%),
    linear-gradient(180deg, 
      #000000 0%, 
      #050a14 15%,
      #0a1428 30%,
      #0f1e3c 45%,
      #142850 60%,
      #193264 75%,
      #1e3c78 90%,
      #2659FF 100%
    );
  border: 4px solid #000000;
  border-radius: 8px;
  box-shadow: 
    0 0 0 8px #000000,
    0 15px 50px rgba(0, 0, 0, 0.9),
    inset 0 0 60px rgba(255, 255, 255, 0.05);
  overflow: hidden;
}

/* Rows and columns */
.board-row {
  display: flex;
  position: absolute;
  left: 0;
  right: 0;
}

.top-row {
  top: 0;
  height: var(--corner-size);
}

.bottom-row {
  bottom: 0;
  height: var(--corner-size);
}

.board-middle {
  position: absolute;
  top: var(--corner-size);
  bottom: var(--corner-size);
  left: 0;
  right: 0;
  display: flex;
}

.spaces-row {
  display: flex;
  flex: 1;
}

.spaces-column {
  display: flex;
  flex-direction: column;
  width: var(--corner-size);
}

.left-column {
  flex-direction: column-reverse;
}

/* Corner spaces - consistent sizing */
.corner-space {
  width: var(--corner-size);
  height: var(--corner-size);
  background: linear-gradient(135deg, #f8f9fa 0%, #e9ecef 100%);
  border: 2px solid #000000;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  box-sizing: border-box;
  position: relative;
}

.corner-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  width: 100%;
  height: 100%;
  padding: 4px;
  box-sizing: border-box;
}

.corner-icon {
  font-size: calc(var(--corner-size) * 0.28);
  line-height: 1;
}

.corner-label {
  font-family: 'Oswald', sans-serif;
  font-size: calc(var(--corner-size) * 0.11);
  font-weight: 700;
  color: #0D47A1;
  line-height: 1.1;
  text-transform: uppercase;
  margin-top: 2px;
}

/* START corner */
.corner-content.start {
  background: linear-gradient(135deg, #ff6b6b 0%, #ee5a5a 100%);
}

.start-diamond-final {
  width: calc(var(--corner-size) * 0.3);
  height: calc(var(--corner-size) * 0.3);
  background: #1976D2;
  transform: rotate(45deg);
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 4px 10px rgba(0,0,0,0.25);
  margin-bottom: 8px;
  border-radius: 6px;
  border: 2px solid white;
}

.start-arrow-final {
  font-size: calc(var(--corner-size) * 0.2);
  color: white;
  transform: rotate(135deg); /* Points Left */
  line-height: 1;
}

.start-label {
  color: white;
  font-size: calc(var(--corner-size) * 0.16);
  text-shadow: 1px 1px 2px rgba(0,0,0,0.3);
}

.start-bonus {
  font-family: 'Roboto', sans-serif;
  font-size: calc(var(--corner-size) * 0.08);
  color: white;
  margin-top: 2px;
}

.tokens-at-start {
  position: absolute;
  top: 48%; /* Slightly above center to avoid covering bonus text */
  left: 45%; /* Shifted left as requested */
  transform: translate(-50%, -50%) scale(0.9);
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  grid-template-rows: repeat(2, 1fr);
  grid-auto-flow: column; /* This puts player 1 & 2 in left column, 3 & 4 in right */
  width: 65px;
  height: 65px;
  gap: 2px 10px; /* Big horizontal gap between columns */
  pointer-events: none;
  z-index: 25;
  align-items: center;
  justify-items: center;
}

.tokens-at-start > * {
  pointer-events: auto;
}

/* JUST VISITING corner */
.corner-content.just-visiting {
  background: #000000;
  position: relative;
}

.jail-box {
  position: absolute;
  top: 4px;
  left: 4px;
  width: calc(var(--corner-size) * 0.58);
  height: calc(var(--corner-size) * 0.58);
  background: #C44601;
  border: 3px solid #000000;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: 'Oswald', sans-serif;
  font-size: calc(var(--corner-size) * 0.13);
  font-weight: 700;
  color: white;
  box-sizing: border-box;
  box-shadow: inset 0 0 15px rgba(0,0,0,0.4);
}

.visiting-label {
  position: absolute;
  bottom: 3px;
  right: 3px;
  font-size: calc(var(--corner-size) * 0.09);
  text-align: right;
}

/* FREE PARKING corner */
.corner-content.free-parking {
  background: linear-gradient(135deg, #a8e6cf 0%, #88d8b0 100%);
}

/* GO TO JAIL corner */
.corner-content.go-to-jail {
  background: linear-gradient(135deg, #ffd93d 0%, #ff9f43 100%);
}

/* Regular spaces */
.space {
  width: var(--space-width);
  height: var(--corner-size);
  background: linear-gradient(180deg, #f8f9fa 0%, #e9ecef 100%);
  border: 1px solid #000000;
  display: flex;
  flex-direction: column;
  position: relative;
  box-sizing: border-box;
}

.space.destination {
  background: linear-gradient(0deg, #000000 0%, #0077FF 100%);
}

/* Side-specific property gradients: Outer Black -> Inner Blue */
.bottom-row .space.destination {
  background: linear-gradient(0deg, #000000 0%, #0077FF 100%);
}

.top-row .space.destination {
  background: linear-gradient(180deg, #000000 0%, #0077FF 100%);
}

.left-column .space.destination {
  background: linear-gradient(90deg, #000000 0%, #0077FF 100%);
}

.right-column .space.destination {
  background: linear-gradient(270deg, #000000 0%, #0077FF 100%);
}

.horizontal-space {
  width: var(--corner-size);
  height: var(--space-width);
  flex-direction: row;
}

.horizontal-space .color-band {
  width: 22%;
  height: 100%;
}

.left-side .color-band {
  order: 1;
}

.right-side .color-band {
  order: -1;
}

.bottom-space {
  flex-direction: column-reverse;
}

.bottom-space .color-band {
  order: 1;
}

/* Color bands */
.color-band {
  width: 100%;
  height: 24%;
  flex-shrink: 0;
}

/* Space content */
.space-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 2px;
  text-align: center;
  position: relative; /* Needed for absolute special wrappers */
}

.horizontal-space .space-content {
  writing-mode: vertical-rl;
  text-orientation: mixed;
}

/* Left side spaces: Text should face left (outwards) - Default vertical-rl has base to left */
.left-side .space-content {
  transform: none;
}

/* Right side spaces: Text should face right (outwards) - Needs 180 rotation from base-left */
.right-side .space-content {
  transform: rotate(180deg);
}

.top-row .space {
  flex-direction: column-reverse; /* Color band at bottom (interior) */
}

.top-row .space-content {
  transform: rotate(180deg);
}

/* Property Stamp Overlay - perfectly centered on the color bar */
.property-stamp {
  position: absolute;
  z-index: 10;
  pointer-events: none;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* Positioning for vertical fields (top/bottom) */
/* Centered on the border with a slight 6px interior offset logic */
.top-row .property-stamp {
  bottom: 6px; /* Anchor 6px inside, translate 50% down (towards center) */
  left: 50%;
  transform: translate(-50%, 50%) scale(0.24) rotate(180deg);
}
.bottom-row .property-stamp {
  top: 6px; /* Anchor 6px inside, translate 50% up (towards center) */
  left: 50%;
  transform: translate(-50%, -50%) scale(0.24);
}

/* Positioning for horizontal fields (left/right) */
/* Aligned to the interior edge to allow border overlap */
.left-column .property-stamp {
  right: 6px;
  top: 52%;
  transform: translate(50%, -50%) scale(0.24) rotate(90deg);
}
.right-column .property-stamp {
  left: 6px;
  top: 52%;
  transform: translate(-50%, -50%) scale(0.24) rotate(-90deg);
}

.space-name {
  font-family: 'Oswald', sans-serif;
  font-size: calc(var(--space-width) * 0.14);
  font-weight: 600;
  color: white;
  line-height: 1.1;
  text-transform: uppercase;
}

.space-price {
  font-family: 'Roboto', sans-serif;
  font-size: calc(var(--space-width) * 0.11);
  color: #a0c0ff;
  font-weight: 500;
}

.space:not(.destination) .space-name {
  color: #0D47A1;
}

.space:not(.destination) .space-price {
  color: #1565C0;
}



/* Special space types */
.space.chance {
  background: linear-gradient(135deg, #e3f2fd 0%, #bbdefb 100%);
}

.space.airport {
  background: linear-gradient(135deg, #fff8e1 0%, #ffecb3 100%);
}

.space.here_and_now {
  background: linear-gradient(135deg, #e8f5e9 0%, #c8e6c9 100%);
}

.space.first_class {
  background: linear-gradient(135deg, #f3e5f5 0%, #e1bee7 100%);
}

.space-icon {
  font-size: calc(var(--space-width) * 0.22);
  margin-bottom: 2px;
}

.space-type-label {
  font-family: 'Oswald', sans-serif;
  font-size: calc(var(--space-width) * 0.085);
  font-weight: 500;
  color: #37474f;
  text-transform: uppercase;
}

/* Special Space Redesigns */
.space.first_class {
  background: radial-gradient(circle at center, #f5f5f5 0%, #e0e0e0 100%);
}

/* Standardized Special Content Wrapper */
.special-content-wrapper {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  position: relative;
  writing-mode: horizontal-tb !important;
  overflow: visible; /* Ensure nothing is cut off */
}

/* Specific rotations for left/right special spaces content - use square wrapping to avoid clipping */
.left-side .special-content-wrapper,
.right-side .special-content-wrapper {
  position: absolute;
  top: 50%;
  left: 50%;
  width: var(--corner-size);
  height: var(--corner-size);
  z-index: 5;
}

.left-side .special-content-wrapper {
  transform: translate(-50%, -50%) rotate(90deg);
}

.right-side .special-content-wrapper {
  transform: translate(-50%, -50%) rotate(90deg);
}

.first-class-content, .airport-content, .here-now-content, .chance-content {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 4px;
  box-sizing: border-box;
  gap: 2px;
}

.special-label {
  font-family: 'Oswald', sans-serif;
  font-size: calc(var(--space-width) * 0.09);
  font-weight: 700;
  color: #333;
  text-transform: uppercase;
}

.special-price {
  font-family: 'Roboto', sans-serif;
  font-size: calc(var(--space-width) * 0.08);
  font-weight: 700;
  color: #333;
}

/* Elegant First Class Logo v2 */
.elegant-star-ring-v2 {
  width: calc(var(--space-width) * 0.5);
  height: calc(var(--space-width) * 0.5);
  border: 1.5px solid #FFD700;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: white;
  position: relative;
  box-shadow: 0 0 8px rgba(255, 215, 0, 0.2);
}

.elegant-star-ring-v2::after {
  content: '';
  position: absolute;
  inset: 2px;
  border: 1px solid #FFD54F;
  border-radius: 50%;
}

.elegant-star-v2 {
  font-size: calc(var(--space-width) * 0.3);
  color: #FFD700;
  line-height: 1;
  z-index: 1;
}

.space.first_class {
  background: radial-gradient(circle at center, #ffffff 0%, #e0e0e0 100%);
}

/* Airport simple icon */
.space.airport {
  background: #03A9F4;
}

.space.airport .special-label, .space.airport .special-price {
  color: white;
}

.airport-emoji {
  font-size: calc(var(--space-width) * 0.4);
  filter: drop-shadow(0 0 2px rgba(0,0,0,0.2));
}

/* Here & Now (Preserved) */
.space.here_and_now {
  background: #cc0000;
}

.globe-icon-outline {
  font-size: calc(var(--space-width) * 0.35);
}

.here-now-text {
  display: flex;
  flex-direction: column;
  align-items: center;
  line-height: 1;
}

.here-now-text span {
  font-family: 'Oswald', sans-serif;
  color: white;
  font-weight: 700;
  text-transform: uppercase;
}

.text-here { font-size: calc(var(--space-width) * 0.14); }
.text-and { font-size: calc(var(--space-width) * 0.1); color: #ddd; }
.text-now { font-size: calc(var(--space-width) * 0.18); }

/* Chance (Preserved) */
.space.chance {
  background: #9c27b0;
}

.large-question-mark {
  font-family: 'Oswald', sans-serif;
  font-size: calc(var(--space-width) * 0.6);
  color: white;
  font-weight: 700;
  line-height: 1;
}

.chance-label {
  font-family: 'Oswald', sans-serif;
  font-size: calc(var(--space-width) * 0.1);
  color: white;
  font-weight: 400;
  letter-spacing: 1px;
}

/* Board center - Rigid container for passports */
.board-center {
  flex: 1;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden; /* CRITICAL: Clips anything that tries to overlap towns */
  background: transparent;
}

/* Passport zones in 4 corners of center - No background, rotated */
.passport-zone {
  position: absolute;
  width: 0;
  height: 0;
  z-index: 5;
}

.zone-top-left {
  top: 25px;
  left: 25px;
}
.zone-top-left .passport-area {
  position: absolute;
  bottom: 0;
  left: 0;
  transform: translateX(-50%) scale(0.32) rotate(135deg);
  transform-origin: bottom center;
}

.zone-top-right {
  top: 25px;
  right: 25px;
}
.zone-top-right .passport-area {
  position: absolute;
  bottom: 0;
  right: 0;
  transform: translateX(50%) scale(0.32) rotate(-135deg); /* +50% because right-aligned */
  transform-origin: bottom center;
}

.zone-bottom-left {
  bottom: 25px;
  left: 25px;
}
.zone-bottom-left .passport-area {
  position: absolute;
  bottom: 0;
  left: 0;
  transform: translateX(-50%) scale(0.32) rotate(45deg);
  transform-origin: bottom center;
}

.zone-bottom-right {
  bottom: 25px;
  right: 25px;
}
.zone-bottom-right .passport-area {
  position: absolute;
  bottom: 0;
  right: 0;
  transform: translateX(50%) scale(0.32) rotate(-45deg); /* +50% because right-aligned */
  transform-origin: bottom center;
}

.passport-area {
  box-shadow: 0 10px 25px rgba(0,0,0,0.4);
}

/* Decks in center */
.card-deck {
  position: absolute;
  width: calc(var(--board-size) * 0.12);
  height: calc(var(--board-size) * 0.18);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2;
}

.chance-deck {
  top: 50%;
  left: 18%;
  transform: translate(-50%, -50%);
}

.here-now-deck {
  top: 50%;
  right: 18%;
  transform: translate(50%, -50%);
}

.deck-outline {
  width: 100%;
  height: 100%;
  border: 2px dashed #ff5252;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent; /* Cleaner without background */
}

.dice-container-center {
  z-index: 10;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  pointer-events: auto;
  transform: translateY(-85px); /* Main dice shift */
  position: relative; /* Anchor for absolute log */
}

.activity-log-pos {
  position: absolute;
  top: 100%;
  margin-top: 40px; /* Gap to move it even lower as requested */
}

.dice-control-panel {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  background: rgba(0, 0, 0, 0.7);
  padding: 16px 20px;
  border-radius: 16px;
  border: 2px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.8);
  backdrop-filter: blur(10px);
}

.forced-deal-panel {
  padding: 14px;
  gap: 12px;
  max-width: 180px; /* Matching DicePanel aesthetic width */
  min-width: 160px;
}

.forced-deal-panel h3 {
  font-size: 1.1rem;
  font-weight: 900;
  margin: 0;
  color: #fbbf24;
  text-transform: uppercase;
  letter-spacing: 2px;
}

.forced-deal-panel p {
  font-size: 0.8rem;
  font-weight: 600;
  margin: -4px 0 0 0;
  color: #94a3b8;
}

.forced-deal-panel .modal-btn {
  padding: 10px 16px;
  font-size: 14px;
  font-weight: 800;
  letter-spacing: 1px;
}

.forced-deal-panel .modal-buttons.vertical {
  width: 100%;
}

.modal-btn.sneaky {
  background: linear-gradient(135deg, #818cf8 0%, #6366f1 100%);
  color: white;
  box-shadow: 0 4px 15px rgba(99, 102, 241, 0.4);
}

.modal-btn.move {
  background: linear-gradient(135deg, #f472b6 0%, #ec4899 100%);
  color: white;
  box-shadow: 0 4px 15px rgba(236, 72, 153, 0.4);
}

.forced-deal-panel .target-btn {
  padding: 4px;
}

.forced-deal-panel .target-token {
  width: 24px;
  height: 24px;
}

.forced-deal-panel .target-selection-grid {
  gap: 8px;
}

.roll-button {
  background: linear-gradient(135deg, #ff6b6b 0%, #ee5a5a 100%);
  border: none;
  border-radius: 12px;
  padding: 10px 24px;
  font-family: 'Oswald', sans-serif;
  font-size: 18px;
  font-weight: 700;
  color: white;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: all 0.3s ease;
  box-shadow: 0 4px 15px rgba(255, 107, 107, 0.4);
  text-transform: uppercase;
  letter-spacing: 1px;
}

.roll-button:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(255, 107, 107, 0.6);
  background: linear-gradient(135deg, #ff7b7b 0%, #ff6a6a 100%);
}

.roll-button:active:not(:disabled) {
  transform: translateY(0);
}

.roll-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.roll-icon {
  font-size: 22px;
  line-height: 1;
}

.roll-text {
  line-height: 1;
}

.turn-indicator {
  font-family: 'Roboto', sans-serif;
  font-size: 14px;
  color: #fff;
  background: rgba(255, 255, 255, 0.1);
  padding: 6px 12px;
  border-radius: 8px;
  text-align: center;
  font-weight: 500;
}

/* Jail Panel */
.jail-panel {
  min-width: 240px; /* Smaller as requested */
  padding: 16px;
  gap: 12px;
  border: 2px solid #ef4444;
  margin-top: 100px; /* Moved lower */
}

.jail-header h3 {
  font-family: 'Oswald', sans-serif;
  font-size: 1.2rem; /* Slightly smaller */
  color: #ef4444;
  margin: 0;
  letter-spacing: 2px;
}

.jail-header p {
  color: #94a3b8;
  font-size: 0.9rem;
  margin: 4px 0 0 0;
}

.jail-options {
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: 100%;
}

.jail-opt {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 10px 15px;
  border-radius: 12px;
  transition: all 0.2s ease;
}

.jail-opt .opt-label {
  font-size: 1rem;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 1px;
}

.jail-opt .opt-desc {
  font-size: 0.7rem;
  font-weight: 500;
  opacity: 0.8;
}

.jail-opt.pay {
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
  color: white;
  box-shadow: 0 4px 12px rgba(16, 185, 129, 0.3);
}

.jail-opt.card {
  background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
  color: white;
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
}

.jail-opt.roll {
  background: linear-gradient(135deg, #6366f1 0%, #4f46e5 100%);
  color: white;
  box-shadow: 0 4px 12px rgba(99, 102, 241, 0.3);
}

.jail-opt:disabled {
  filter: grayscale(1);
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}

/* Forced Deal Modal */
.forced-deal-modal {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.4); /* Reduced opacity, no blur */
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

.modal-content {
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
  padding: 32px;
  border-radius: 20px;
  border: 3px solid #ffd700;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.9);
  text-align: center;
  animation: slideIn 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}

@keyframes slideIn {
  from { transform: scale(0.8) translateY(-30px); opacity: 0; }
  to { transform: scale(1) translateY(0); opacity: 1; }
}

.modal-content h3 {
  font-family: 'Oswald', sans-serif;
  font-size: 28px;
  color: #ffd700;
  margin: 0 0 12px 0;
  text-transform: uppercase;
  letter-spacing: 2px;
}

.modal-content p {
  font-family: 'Roboto', sans-serif;
  font-size: 16px;
  color: #fff;
  margin: 0 0 24px 0;
}

.modal-buttons {
  display: flex;
  gap: 16px;
  justify-content: center;
}

.modal-btn {
  font-family: 'Oswald', sans-serif;
  font-size: 16px;
  font-weight: 600;
  padding: 12px 24px;
  border: none;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.3s ease;
  text-transform: uppercase;
  letter-spacing: 1px;
}

.modal-btn.sneaky {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  box-shadow: 0 4px 15px rgba(102, 126, 234, 0.4);
}

.modal-btn.sneaky:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(102, 126, 234, 0.6);
}

.modal-btn.move {
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
  color: white;
  box-shadow: 0 4px 15px rgba(245, 87, 108, 0.4);
}

.modal-btn.move:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(245, 87, 108, 0.6);
}

.forced-deal-panel {
  padding: 16px !important;
  width: 240px !important;
}

.forced-deal-panel h3 {
  font-size: 18px !important;
  margin-bottom: 4px !important;
}

.forced-deal-panel p {
  font-size: 13px !important;
  margin-bottom: 12px !important;
}

.forced-deal-panel .modal-btn {
  padding: 10px 16px;
  font-size: 14px;
}

.modal-buttons.vertical {
  flex-direction: column;
  width: 100%;
  gap: 8px;
}

.target-selection-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 10px;
  width: 100%;
  margin-bottom: 15px;
}

.target-btn {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 8px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.target-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.15);
  border-color: #ffd700;
  transform: translateY(-2px);
}

.target-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
  filter: grayscale(1);
}

.target-token {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.target-name {
  font-size: 11px;
  color: #fff;
  font-weight: 600;
  text-transform: uppercase;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.modal-btn.skip.mini {
  padding: 6px 16px;
  font-size: 12px;
}

/* Purchase and First Class modals - reuse forced-deal-modal base */
.purchase-modal,
.first-class-modal,
.airport-modal {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: 1000;
  animation: fadeIn 0.3s ease-out;
}

.modal-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.purchase-modal .modal-content,
.first-class-modal .modal-content,
.airport-modal .modal-content {
  background: linear-gradient(145deg, #ffffff 0%, #f0f0f0 100%);
  border-radius: 20px;
  padding: 24px 32px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  border: 3px solid #ffd700;
  text-align: center;
}

.modal-content.airport {
  padding: 16px 24px;
  min-width: 200px;
}

.modal-content.airport h3 {
  font-size: 20px;
  margin-bottom: 8px;
}

.modal-content.airport p {
  font-size: 14px;
  margin-bottom: 16px;
}

.purchase-modal h3,
.first-class-modal h3 {
  font-family: 'Oswald', sans-serif;
  font-size: 24px;
  color: #1976D2;
  margin-bottom: 12px;
}

.property-name {
  font-family: 'Oswald', sans-serif;
  font-size: 20px;
  color: #333;
  font-weight: 600;
}

.property-price {
  font-family: 'Roboto', sans-serif;
  font-size: 18px;
  color: #4CAF50;
  font-weight: 700;
  margin-bottom: 16px;
}

.modal-btn.buy {
  background: linear-gradient(135deg, #4CAF50 0%, #2E7D32 100%);
  color: white;
  box-shadow: 0 4px 15px rgba(76, 175, 80, 0.4);
}

.modal-btn.buy:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(76, 175, 80, 0.6);
}

.modal-btn.skip {
  background: linear-gradient(135deg, #9E9E9E 0%, #616161 100%);
  color: white;
  box-shadow: 0 4px 15px rgba(158, 158, 158, 0.4);
}

.modal-btn.skip:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(158, 158, 158, 0.6);
}

/* Positioned Tokens */
.positioned-token {
  position: absolute;
  pointer-events: auto;
  transition: top 0.4s cubic-bezier(0.25, 0.46, 0.45, 0.94), 
              left 0.4s cubic-bezier(0.25, 0.46, 0.45, 0.94);
  transform: translate(-50%, -50%);
}

/* Responsive adjustments */
@media (max-width: 600px) {
  .space-name {
    font-size: calc(var(--space-width) * 0.12);
  }
  
  .space-price {
    font-size: calc(var(--space-width) * 0.1);
  }
  
  .corner-label {
    font-size: calc(var(--corner-size) * 0.09);
  }
  
  .passport-area {
    transform: scale(0.25);
  }
}

/* Airport Selection */
.space.selectable-destination {
  cursor: pointer;
  position: relative;
  overflow: visible !important;
}

.space.selectable-destination::after {
  content: '';
  position: absolute;
  top: -4px;
  left: -4px;
  right: -4px;
  bottom: -4px;
  border: 4px solid #fff;
  border-radius: 8px;
  box-shadow: 0 0 15px rgba(255, 255, 255, 0.8);
  animation: pulse-border 1.5s infinite;
  pointer-events: none;
  z-index: 100;
}

@keyframes pulse-border {
  0% { transform: scale(1); opacity: 0.6; }
  50% { transform: scale(1.05); opacity: 1; }
  100% { transform: scale(1); opacity: 0.6; }
}

.airport-modal .modal-icon {
  font-size: 3rem;
  margin-bottom: 0.5rem;
  animation: float 3s ease-in-out infinite;
}

.airport-modal .modal-icon.mini {
  font-size: 2rem;
  margin-bottom: 0.2rem;
}

@keyframes float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-10px); }
}
/* Auction Modal Styles */
.auction-modal {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: 2000; /* Higher than purchase modal */
  animation: bounceIn 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}

.modal-content.auction {
  background: linear-gradient(145deg, #1a1c20 0%, #2d3436 100%);
  border: 3px solid #ff9f43;
  color: white;
  min-width: 320px;
  padding: 24px;
}

.auction-header {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  margin-bottom: 8px;
}

.auction-icon {
  font-size: 28px;
  animation: swing 1s infinite ease-in-out;
}

.modal-content.auction h3 {
  color: #ff9f43;
  font-size: 28px;
  margin: 0;
  text-shadow: 0 2px 4px rgba(0,0,0,0.5);
}

.modal-content.auction .property-name {
  color: white;
  margin-bottom: 20px;
  font-size: 22px;
}

.auction-stats {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 20px;
  background: rgba(0,0,0,0.3);
  padding: 12px;
  border-radius: 12px;
}

.stat-box {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.stat-label {
  font-size: 12px;
  color: #a4b0be;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.stat-value {
  font-family: 'Oswald', sans-serif;
  font-size: 18px;
  font-weight: 600;
}

.bid-amount { color: #2ed573; }
.bidder-name { color: #70a1ff; }

.auction-timer-container {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 24px;
}

.auction-timer-bar {
  flex: 1;
  height: 12px;
  background: rgba(255,255,255,0.1);
  border-radius: 6px;
  overflow: hidden;
}

.timer-fill {
  height: 100%;
  background: linear-gradient(90deg, #ff4757, #ff6b81);
  transition: width 1s linear;
}

.timer-text {
  font-family: 'Oswald', sans-serif;
  font-size: 20px;
  font-weight: 700;
  color: #ff4757;
  width: 30px;
  text-align: right;
}

.auction-buttons {
  display: flex;
  justify-content: center;
  gap: 12px;
}

.modal-btn.bid {
  background: linear-gradient(135deg, #3742fa 0%, #5352ed 100%);
  color: white;
  min-width: 70px;
  font-size: 18px;
}

.modal-btn.bid:hover:not(:disabled) {
  transform: translateY(-3px);
  box-shadow: 0 8px 15px rgba(55, 66, 250, 0.4);
}

.modal-btn.bid:disabled {
  background: #57606f;
  color: #a4b0be;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}

@keyframes swing {
  0% { transform: rotate(0deg); }
  20% { transform: rotate(15deg); }
  40% { transform: rotate(-10deg); }
  60% { transform: rotate(5deg); }
  80% { transform: rotate(-5deg); }
  100% { transform: rotate(0deg); }
}

@keyframes bounceIn {
  0% { transform: translate(-50%, -50%) scale(0.3); opacity: 0; }
  50% { transform: translate(-50%, -50%) scale(1.05); opacity: 1; }
  70% { transform: translate(-50%, -50%) scale(0.9); }
  100% { transform: translate(-50%, -50%) scale(1); }
}
</style>
/* Auction Modal Styles */
.auction-modal {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: 2000; /* Higher than purchase modal */
  animation: bounceIn 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}

.modal-content.auction {
  background: linear-gradient(145deg, #1a1c20 0%, #2d3436 100%);
  border: 3px solid #ff9f43;
  color: white;
  min-width: 320px;
  padding: 24px;
}

.auction-header {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  margin-bottom: 8px;
}

.auction-icon {
  font-size: 28px;
  animation: swing 1s infinite ease-in-out;
}

.modal-content.auction h3 {
  color: #ff9f43;
  font-size: 28px;
  margin: 0;
  text-shadow: 0 2px 4px rgba(0,0,0,0.5);
}

.modal-content.auction .property-name {
  color: white;
  margin-bottom: 20px;
  font-size: 22px;
}

.auction-stats {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 20px;
  background: rgba(0,0,0,0.3);
  padding: 12px;
  border-radius: 12px;
}

.stat-box {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.stat-label {
  font-size: 12px;
  color: #a4b0be;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.stat-value {
  font-family: 'Oswald', sans-serif;
  font-size: 18px;
  font-weight: 600;
}

.bid-amount { color: #2ed573; }
.bidder-name { color: #70a1ff; }

.auction-timer-container {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 24px;
}

.auction-timer-bar {
  flex: 1;
  height: 12px;
  background: rgba(255,255,255,0.1);
  border-radius: 6px;
  overflow: hidden;
}

.timer-fill {
  height: 100%;
  background: linear-gradient(90deg, #ff4757, #ff6b81);
  transition: width 1s linear;
}

.timer-text {
  font-family: 'Oswald', sans-serif;
  font-size: 20px;
  font-weight: 700;
  color: #ff4757;
  width: 30px;
  text-align: right;
}

.auction-buttons {
  display: flex;
  justify-content: center;
  gap: 12px;
}

.modal-btn.bid {
  background: linear-gradient(135deg, #3742fa 0%, #5352ed 100%);
  color: white;
  min-width: 70px;
  font-size: 18px;
}

.modal-btn.bid:hover:not(:disabled) {
  transform: translateY(-3px);
  box-shadow: 0 8px 15px rgba(55, 66, 250, 0.4);
}

.modal-btn.bid:disabled {
  background: #57606f;
  color: #a4b0be;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}

@keyframes swing {
  0% { transform: rotate(0deg); }
  20% { transform: rotate(15deg); }
  40% { transform: rotate(-10deg); }
  60% { transform: rotate(5deg); }
  80% { transform: rotate(-5deg); }
  100% { transform: rotate(0deg); }
}

@keyframes bounceIn {
  0% { transform: translate(-50%, -50%) scale(0.3); opacity: 0; }
  50% { transform: translate(-50%, -50%) scale(1.05); opacity: 1; }
  70% { transform: translate(-50%, -50%) scale(0.9); }
  100% { transform: translate(-50%, -50%) scale(1); }
}
