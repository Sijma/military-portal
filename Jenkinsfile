pipeline {
  agent any
  environment {
    IMAGE = 'ghcr.io/sijma/military-portal-backend'
    CARGO_BIN  = 'military-portal'
    BINARY_NAME = 'military-portal-linux-x86_64'
    REPO = 'Sijma/military-portal'
    GH_TOKEN = credentials('github-pet')
    CACHE_DIR = '/var/lib/jenkins/cache/rust'
  }
  stages {
    stage('Test and build') {
      steps {
        sh '''
          set -eu
          mkdir -p "$CACHE_DIR/registry" "$CACHE_DIR/target"
          docker run --rm \
            -v "$WORKSPACE":/w -w /w \
            -v "$CACHE_DIR/registry":/cargo \
            -v "$CACHE_DIR/target":/target \
            -e CARGO_HOME=/cargo \
            -e CARGO_TARGET_DIR=/target \
            -u "$(id -u):$(id -g)" \
            rust:1.98.0-alpine \
            sh -c 'cargo test --release --locked && cargo build --release --locked'
          mkdir -p dist
          cp "$CACHE_DIR/target/release/$CARGO_BIN" "dist/$BINARY_NAME"
          test -x "dist/$BINARY_NAME"
        '''
      }
    }

    stage('Publish image') {
      when {
          buildingTag()
      }
      steps {
        sh '''
          set -eu
          echo "$GH_TOKEN" | docker login ghcr.io -u sijma --password-stdin
          docker build -t "$IMAGE:$TAG_NAME" -t "$IMAGE:latest" .
          docker push "$IMAGE:$TAG_NAME"
          docker push "$IMAGE:latest"
        '''
      }
    }

    stage('Publish binary') {
      when { buildingTag() }
      steps {
        createGitHubRelease(
          credentialId: 'github-pet',
          repository: env.REPO,
          tag: env.TAG_NAME,
          name: "Release ${env.TAG_NAME}",
          bodyText: "Automated release build for ${env.TAG_NAME}."
        )

        uploadGithubReleaseAsset(
          credentialId: 'github-pet',
          repository: env.REPO,
          tagName: env.TAG_NAME,
          uploadAssets: [[filePath: "dist/${env.BINARY_NAME}"]]
        )
      }
    }
  }

  post {
    always {
      sh 'docker logout ghcr.io || true'
    }
  }
}
