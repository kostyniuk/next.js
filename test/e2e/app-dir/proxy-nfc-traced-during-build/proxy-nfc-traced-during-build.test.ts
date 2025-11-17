import { nextTestSetup } from 'e2e-utils'

// This test verifies an edge case where the compiled "proxy.js" file is being imported
// during build and being traced into NFT. During build, tools like Sentry wrap user code
// in templates and import the compiled "proxy.js" source, and generate a source map.
// This causes Next.js to trace "proxy.js" into the NFT file. However, since we rename 
// "proxy.js" to "middleware.js", the files in NFT will differ from the actual outputs,
// which will fail for the providers like Vercel that uses NFT. This test verifies that
// we rename the "proxy.js" to "middleware.js" in the NFT file to make it consistent with
// the actual outputs.

describe('proxy-nfc-traced-during-build', () => {
  const { next } = nextTestSetup({
    files: __dirname,
    // This only occurs in Webpack builds, which is handled in Webpack loader.
    buildCommand: 'next build --webpack',
    dependencies: {
      // Fix the version of Sentry to persist the behavior.
      '@sentry/nextjs': '10.25.0',
    },
  })

  it('should successfully build and be redirected from proxy', async () => {
    const $ = await next.render$('/home')
    expect($('p').text()).toBe('hello world')
  })
})
