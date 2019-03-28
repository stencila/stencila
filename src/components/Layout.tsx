import { Column } from 'rbx'
import 'rbx/index.css'
import * as React from 'react'
import { HeaderMenu } from './HeaderMenu'
import { SidebarMenu } from './SidebarMenu'

export interface LayoutProps {
  location: {
    pathname: string
  }
  children: any
}

export const Layout = (props: LayoutProps) => (
  <>
    <HeaderMenu />

    <div style={{ padding: '1rem' }}>
      <Column.Group>
        <SidebarMenu />

        {props.children}
      </Column.Group>
    </div>
  </>
)

export const withLayout = <P extends object>(
  WrappedComponent: React.ComponentType<P>
) =>
  class WithLayout extends React.Component<P & LayoutProps> {
    render() {
      return (
        <Layout location={this.props.location}>
          <WrappedComponent {...this.props} />
        </Layout>
      )
    }
  }
