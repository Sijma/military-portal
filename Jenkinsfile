pipeline {
  agent any
  environment {
    IMAGE = 'ghcr.io/sijma/military-portal-backend'
    BINARY_NAME = 'military-portal-backend'
    REPO  = 'Sijma/military-portal'
    TOKEN = credentials('github-pet')
  }
  stages {
    stage('Build') {
      agent { docker { image 'rust:1.98.0-alpine'; reuseNode true } }
      steps {
        sh 'cargo build --release --locked'
        sh 'file target/release/military-portal || true'
      }
    }

    stage('Publish image') {
      when {
          buildingTag()
      }
      steps {
          def imageTag = "${IMAGE}:${TAG_NAME}"
          sh '''
            set -eu
            echo "TOKEN" | docker login ghcr.io -u "sijma" --password-stdin
            docker build -t "$IMAGE:$TAG_NAME" -t "$IMAGE:latest" .
            docker push "$IMAGE:$TAG_NAME"
            docker push "$IMAGE:latest"
          '''
          def customImage = docker.build(imageTag)
          customImage.push()
          customImage.push('latest')
        }
      }
    stage('Publish binary') {
      when { buildingTag() }
      steps {
        withCredentials([string(credentialsId: 'github-token', variable: 'GH_TOKEN')]) {
          sh '''
            gh release create "${TAG_NAME}" \
                "./target/release/${BINARY_NAME}#${BINARY_NAME}-linux-x86_64" \
                --repo "${REPO}" \
                --title "Release ${TAG_NAME}" \
                --notes "Automated release build for ${TAG_NAME}"
        '''
        }
      }
    }
  }
}


post {
always { sh 'docker logout ghcr.io || true' }
}
