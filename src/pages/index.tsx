import { graphql, useStaticQuery } from 'gatsby'
import { MDXRenderer } from 'gatsby-mdx'
import { Content } from 'rbx'
import * as React from 'react'
import { withLayout } from '../components/Layout'

interface Props {
  data: {
    mdx: {
      code: {
        body: string
      }
    }
  }
}

const IndexPage = () => {
  const data = useStaticQuery(homepageQuery)

  return (
    <Content style={{ padding: '2rem' }}>
      <MDXRenderer>{data.mdx.code.body}</MDXRenderer>
    </Content>
  )
}

export default withLayout(IndexPage)

const homepageQuery = graphql`
  query HomepageQuery {
    mdx(fileAbsolutePath: { regex: "/README/i" }) {
      code {
        body
        scope
      }
    }
  }
`
