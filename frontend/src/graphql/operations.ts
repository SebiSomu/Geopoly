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
      id
      code
      players {
        username
        character
      }
    }
  }
`

export const GET_LOBBY_QUERY = gql`
  query GetLobby($code: String!) {
    getLobby(code: $code) {
      id
      code
      state
      players {
        username
        character
      }
    }
  }
`
