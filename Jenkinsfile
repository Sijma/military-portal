pipeline {
  agent any
  environment {
    IMAGE = 'ghcr.io/sijma/military-portal-backend'
    CARGO_BIN  = 'military-portal'
    BINARY_NAME = 'military-portal-linux-x86_64'
    REPO = 'Sijma/military-portal'
  }
  stages {
    stage('Build') {
      agent {
        docker {
          image 'rust:1.98.0-alpine'
          reuseNode true
          args '-v /var/lib/jenkins/cache/rust/registry:/cargo ' +
               '-v /var/lib/jenkins/cache/rust/target:/target ' +
               '-e CARGO_HOME=/cargo -e CARGO_TARGET_DIR=/target'
        }
      }
      steps {
        sh 'cargo build --release --locked'
        sh 'mkdir -p dist && cp /target/release/$CARGO_BIN dist/$BINARY_NAME'
      }
    }

    stage('Publish image') {
      when { buildingTag() }
      steps {
        withCredentials([string(credentialsId: 'github-pet', variable: 'REG_TOKEN')]) {
          sh '''
            echo "$REG_TOKEN" | docker login ghcr.io -u sijma --password-stdin
            docker build -t "$IMAGE:$TAG_NAME" -t "$IMAGE:latest" .
            docker push "$IMAGE:$TAG_NAME"
            docker push "$IMAGE:latest"
          '''
        }
      }
    }

    stage('Publish release') {
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
