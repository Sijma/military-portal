pipeline {
  agent any
  environment {
    IMAGE = 'ghcr.io/sijma/military-portal-backend'
    CARGO_BIN  = 'military-portal'
    BINARY_NAME = 'military-portal-linux-x86_64'
    REPO = 'Sijma/military-portal'
    TOKEN = credentials('github-pet')
  }
  stages {
    stage('Build') {
      agent { docker { image 'rust:1.98.0-alpine'; reuseNode true } }
      environment {
        CARGO_HOME = "${WORKSPACE}/.cargo"
      }
      steps {
        sh 'cargo build --release --locked'
        sh 'test -x target/release/$CARGO_BIN'
      }
    }

    stage('Publish image') {
      when {
          buildingTag()
      }
      steps {
        sh '''
          set -eu
          echo "$TOKEN" | docker login ghcr.io -u sijma --password-stdin
          docker build -t "$IMAGE:$TAG_NAME" -t "$IMAGE:latest" .
          docker push "$IMAGE:$TAG_NAME"
          docker push "$IMAGE:latest"
        '''
      }
    }

    stage('Publish binary') {
      when { buildingTag() }
      steps {
        withEnv(["GH_TOKEN=${TOKEN}"]) {
          sh '''
            set -eu
            command -v gh > /dev/null || { echo "gh CLI is not installed on this agent"; exit 1; }
            gh release create "$TAG_NAME" \
             "./target/release/$CARGO_BIN#$BINARY_NAME" \
             --repo "$REPO" \
             --title "Release $TAG_NAME" \
             --notes "Automated release build for $TAG_NAME"
          '''
        }
      }
    }
  }

  post {
    always {
      sh 'docker logout ghcr.io || true'
    }
  }
}
