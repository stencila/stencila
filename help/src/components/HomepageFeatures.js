import React from 'react'
import clsx from 'clsx'
import styles from './HomepageFeatures.module.css'

const FeatureList = [
  {
    title: 'Tutorials',
    link: '/tutorials',
    Svg: require('../../static/img/illustrations/tutorials.svg').default,
    description: (
      <>First steps for learning what Stencila does and how to use it.</>
    ),
  },
  {
    title: 'Guides',
    Svg: require('../../static/img/illustrations/guides.svg').default,
    link: '/guides',
    description: (
      <>
        Step-by-step guides to help you achieve a specific goal. Start here when
        you're trying to get a specific task done.
      </>
    ),
  },
  {
    title: 'Demos',
    Svg: require('../../static/img/illustrations/demos.svg').default,
    link: '/demos',
    description: (
      <>
        Demonstrations of functionality and user experience. Start here for a
        taste of the capabilities of Stencila.
      </>
    ),
  },
  {
    title: 'References',
    Svg: require('../../static/img/illustrations/references.svg').default,
    link: '/references',
    description: (
      <>
        Technical descriptions and references. Most useful when you need
        detailed information about how Stencila works.
      </>
    ),
  },
]

function Feature({ Svg, link, title, description }) {
  return (
    <div className={clsx('col col--6 feature', styles.feature)}>
      <div className="text--center">
        <Svg className={styles.featureSvg} alt={title} />
      </div>
      <div className="text--center padding-horiz--md">
        <h3>
          <a href={link}>{title}</a>
        </h3>
        <p>{description}</p>
      </div>
    </div>
  )
}

export default function HomepageFeatures() {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  )
}
