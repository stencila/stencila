import React from 'react'
import clsx from 'clsx'
import styles from './HomepageFeatures.module.css'

const FeatureList = [
  {
    title: 'Tutorials',
    link: '/docs/tutorials',
    Svg: require('../../static/img/tutorials.svg').default,
    description: (
      <>
        Sed ut perspiciatis, unde omnis iste natus error sit voluptatem
        accusantium doloremque laudantium.
      </>
    ),
  },
  {
    title: 'Guides',
    Svg: require('../../static/img/guides.svg').default,
    link: '/docs/guides',
    description: (
      <>
        Sed ut perspiciatis, unde omnis iste natus error sit voluptatem
        accusantium doloremque laudantium, totam rem aperiam eaque ipsa, quae ab
        illo inventore veritatis et quasi architecto beatae vitae dicta sunt,
        explicabo.
      </>
    ),
  },
  {
    title: 'Demos',
    Svg: require('../../static/img/demos.svg').default,
    link: '/docs/demos',
    description: (
      <>
        Sed ut perspiciatis, unde omnis iste natus error sit voluptatem
        accusantium doloremque laudantium, totam.
      </>
    ),
  },
  {
    title: 'Reference',
    Svg: require('../../static/img/reference.svg').default,
    link: '/docs/reference',
    description: (
      <>
        Sed ut perspiciatis, unde omnis iste natus error sit voluptatem
        accusantium doloremque laudantium, totam rem aperiam eaque ipsa, quae ab
        illo inventore.
      </>
    ),
  },
]

function Feature({ Svg, link, title, description }) {
  return (
    <div className={clsx('col col--4')}>
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
