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
        }
        pendingFirstClass
        pendingAirportDecision
        pendingAirportDestination
        isForcedDeal
        isGameOver
        winnerName
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
        }
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
