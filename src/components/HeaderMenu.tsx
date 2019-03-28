import { Link } from 'gatsby'
import { Container, Navbar } from 'rbx'
import * as React from 'react'

export const HeaderMenu = () => (
  <Container>
    <Navbar>
      <Navbar.Brand>
        <Navbar.Item as={Link} to="/">
          <img
            src="https://stenci.la/img/stencila-logo.svg"
            alt=""
            role="presentation"
            width="112"
            height="28"
          />
        </Navbar.Item>
        <Navbar.Burger />
      </Navbar.Brand>

      <Navbar.Menu>
        <Navbar.Segment align="start">
          <Navbar.Item as={Link} to="/">
            Documentation
          </Navbar.Item>
        </Navbar.Segment>
      </Navbar.Menu>
    </Navbar>
  </Container>
)
