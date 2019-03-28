import { Container, Hero, Title } from 'rbx'
import * as React from 'react'
import { withLayout } from '../components/Layout'

const NotFoundPage = () => (
  <Hero>
    <Hero.Body>
      <Container>
        <Title>No results</Title>
      </Container>
    </Hero.Body>
  </Hero>
)

export default withLayout(NotFoundPage)
