import { ApolloClient, InMemoryCache, createHttpLink, split } from '@apollo/client/core'
import { provideApolloClient } from '@vue/apollo-composable'
import { GraphQLWsLink } from '@apollo/client/link/subscriptions'
import { createClient } from 'graphql-ws'
import { getMainDefinition } from '@apollo/client/utilities'

// HTTP connection to the API
const httpLink = createHttpLink({
    uri: 'http://localhost:8000/graphql',
})

// WebSocket connection to the API
const wsLink = new GraphQLWsLink(
    createClient({
        url: 'ws://localhost:8000/graphql/ws',
    })
)

// Split link based on operation type
const link = split(
    ({ query }) => {
        const definition = getMainDefinition(query)
        return (
            definition.kind === 'OperationDefinition' &&
            definition.operation === 'subscription'
        )
    },
    wsLink,
    httpLink
)

// Cache implementation
const cache = new InMemoryCache()

// Create the apollo client
export const apolloClient = new ApolloClient({
    link: link,
    cache,
})

export function setupApollo() {
    provideApolloClient(apolloClient)
}
