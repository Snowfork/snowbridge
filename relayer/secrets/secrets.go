package secrets

import (
	"context"
	"fmt"

	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/config"
	"github.com/aws/aws-sdk-go-v2/service/secretsmanager"
)

func GetSecretValue(ctx context.Context, secretName string) (string, error) {
    cfg, err := config.LoadDefaultConfig(ctx)
    if err != nil {
        return "", fmt.Errorf("load SDK config, %v", err)
    }

    svc := secretsmanager.NewFromConfig(cfg)
    input := &secretsmanager.GetSecretValueInput{
        SecretId: aws.String(secretName),
    }

    result, err := svc.GetSecretValue(ctx, input)
    if err != nil {
        return "", fmt.Errorf("retrieve secret value, %v", err)
    }

    return *result.SecretString, nil
}
