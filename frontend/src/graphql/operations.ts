import gql from 'graphql-tag'

export const REGISTER_MUTATION = gql`
  mutation Register($username: String!, $password: String!) {
    register(username: $username, password: $password) {
      id
      username
    }
  }
`

export const LOGIN_MUTATION = gql`
  mutation Login($username: String!, $password: String!) {
    login(username: $username, password: $password) {
      id
      username
    }
  }
`

export const CREATE_LOBBY_MUTATION = gql`
  mutation CreateLobby($username: String!) {
    createLobby(username: $username) {
      id
      code
      state
      host
      players {
        username
        character
      }
    }
  }
`

export const JOIN_LOBBY_MUTATION = gql`
  mutation JoinLobby($code: String!, $username: String!) {
    joinLobby(code: $code, username: $username) {
      id
      code
      state
      host
      players {
        username
        character
      }
    }
  }
`

export const SELECT_CHARACTER_MUTATION = gql`
  mutation SelectCharacter($code: String!, $username: String!, $character: String!) {
    selectCharacter(code: $code, username: $username, character: $character) {
      code
      players {
        username
        character
      }
    }
  }
`;

export const START_GAME_MUTATION = gql`
  mutation StartGame($code: String!, $username: String!) {
    startGame(code: $code, username: $username) {
      code
      state
    }
  }
`;

export const GET_LOBBY_QUERY = gql`
  query GetLobby($code: String!) {
    getLobby(code: $code) {
      id
      code
      state
      gameState {
        currentTurnIndex
        lastDie1
        lastDie2
        awaitingAction
        pendingPurchase {
          destId
          destName
          price
          buyerIdx
        }
        pendingFirstClass {
          buyerIdx
        }
        pendingAirportDecision {
          buyerIdx
        }
        pendingAirportDestination {
          buyerIdx
        }
        isForcedDeal
        isGameOver
        winnerName
        targetSelection {
          action
          cardId
          selectorIdx
        }
        diceDuel {
          challengerIdx
          targetIdx
          challengerDie1
          challengerDie2
          targetDie1
          targetDie2
        }
        pendingAuction {
          destId
          destName
          currentBid
          highestBidderIdx
        }
        activityLog {
          playerIdx
          message
        }
        isJailDecision
        isRerollDice
        pendingStampSelection {
          action
          cardId
          selectorIdx
        }
      }
      players {
        username
        character
        position
        inJail
        consecutiveDoubles
        money
        properties {
          name
          color
          diameter
          column
          destinationId
          x
          y
          size
          price
        }
        hereAndNowCards {
          id
          description
        }
        chanceCards {
          id
          description
        }
        canUseSayNo
        canUseDiscount
        canUseIntercept
        canUseCollectTax
        canUseStealFirstClass
        skipNextTurn
      }
    }
  }
`

export const ROLL_DICE_MUTATION = gql`
  mutation RollDice($code: String!, $username: String!) {
    rollDice(code: $code, username: $username) {
      die1
      die2
      isDouble
      isForcedDeal
      newPosition
      wentToJail
      turnEnds
      currentTurnIndex
    }
  }
`

export const RESOLVE_FORCED_DEAL_MUTATION = gql`
  mutation ResolveForcedDeal($code: String!, $username: String!, $action: String!, $target: String) {
    resolveForcedDeal(code: $code, username: $username, action: $action, target: $target) {
      code
    }
  }
`

export const RESOLVE_PURCHASE_MUTATION = gql`
  mutation ResolvePurchase($code: String!, $username: String!, $buy: Boolean!) {
    resolvePurchase(code: $code, username: $username, buy: $buy) {
      code
    }
  }
`

export const RESOLVE_FIRST_CLASS_MUTATION = gql`
  mutation ResolveFirstClass($code: String!, $username: String!, $buy: Boolean!) {
    resolveFirstClass(code: $code, username: $username, buy: $buy) {
      code
    }
  }
`

export const RESOLVE_AIRPORT_DECISION_MUTATION = gql`
  mutation ResolveAirportDecision($code: String!, $username: String!, $buyFlight: Boolean!) {
    resolveAirportDecision(code: $code, username: $username, buyFlight: $buyFlight) {
      code
    }
  }
`

export const RESOLVE_AIRPORT_DESTINATION_MUTATION = gql`
  mutation ResolveAirportDestination($code: String!, $username: String!, $targetPosition: Int!) {
    resolveAirportDestination(code: $code, username: $username, targetPosition: $targetPosition) {
      code
    }
  }
`

export const USE_CARD_MUTATION = gql`
  mutation UseCard($code: String!, $username: String!, $cardId: String!) {
    useCard(code: $code, username: $username, cardId: $cardId) {
      code
    }
  }
`

export const RESOLVE_TARGET_SELECTION_MUTATION = gql`
  mutation ResolveTargetSelection($code: String!, $username: String!, $targetUsername: String!) {
    resolveTargetSelection(code: $code, username: $username, targetUsername: $targetUsername) {
      code
    }
  }
`

export const ROLL_DUEL_DICE_MUTATION = gql`
  mutation RollDuelDice($code: String!, $username: String!) {
    rollDuelDice(code: $code, username: $username) {
      code
    }
  }
`

export const FINISH_DUEL_MUTATION = gql`
  mutation FinishDuel($code: String!, $username: String!) {
    finishDuel(code: $code, username: $username) {
      code
    }
  }
`

export const PLACE_BID_MUTATION = gql`
  mutation PlaceBid($code: String!, $username: String!, $amount: Int!) {
    placeBid(code: $code, username: $username, amount: $amount) {
      code
    }
  }
`

export const RESOLVE_AUCTION_MUTATION = gql`
  mutation ResolveAuction($code: String!) {
    resolveAuction(code: $code) {
      code
    }
  }
`
export const RESOLVE_JAIL_DECISION_MUTATION = gql`
  mutation ResolveJailDecision($code: String!, $username: String!, $action: String!) {
    resolveJailDecision(code: $code, username: $username, action: $action) {
      die1
      die2
      isDouble
      isForcedDeal
      newPosition
      wentToJail
      turnEnds
      currentTurnIndex
    }
  }
`

export const RESOLVE_STAMP_AMNESTY_MUTATION = gql`
  mutation ResolveStampAmnesty($code: String!, $username: String!, $stampName: String!) {
    resolveStampAmnesty(code: $code, username: $username, stampName: $stampName) {
      code
    }
  }
`

export const SEND_MESSAGE_MUTATION = gql`
  mutation SendMessage($code: String!, $sender: String!, $content: String!) {
    sendMessage(code: $code, sender: $sender, content: $content)
  }
`;

export const MESSAGE_RECEIVED_SUBSCRIPTION = gql`
  subscription MessageReceived($code: String!) {
    messageReceived(code: $code) {
      sender
      content
      timestamp
    }
  }
`;
