import { css } from '../../styled-system/css'

interface ResultViewerProps {
	result: {
		same_author: boolean
		confidence: number
		detailed_analysis: Array<{
			aspect: string
			difference: number
			explanation: string
		}>
	}
}

export default function ResultViewer({ result }: ResultViewerProps) {
	console.log(result)

	return (
		<div
			class={css({
				backgroundColor: 'white',
				borderRadius: '12px',
				padding: '24px',
				boxShadow:
					'0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)',
				margin: '24px 0',
			})}
		>
			<h2
				class={css({
					fontSize: '24px',
					fontWeight: 'bold',
					marginBottom: '16px',
					color: '#1a202c',
				})}
			>
				Analysis Result
			</h2>
			<div
				class={css({
					display: 'flex',
					gap: '16px',
					marginBottom: '24px',
					flexWrap: 'wrap',
				})}
			>
				<div
					class={css({
						flex: '1',
						backgroundColor: result.same_author ? '#C6F6D5' : '#FED7D7',
						padding: '16px',
						borderRadius: '8px',
						textAlign: 'center',
						position: 'relative',
						display: 'flex',
						flexDirection: 'column',
						alignItems: 'center',
						gap: '8px'
					})}
				>
						<div
							class={css({
								height: '64px',
								position: 'relative',
								display: 'flex',
								alignItems: 'center',
								justifyContent: 'center',
								zIndex: 1
							})}
						>
							<p
								class={css({
									fontSize: '36px',
									fontWeight: 'bold',
									color: result.same_author ? '#2F855A' : '#C53030',
									position: 'relative',
									zIndex: 1
								})}
							>
								{result.same_author ? 'Written' : 'Not written'}
							</p>
						</div>
					<p
						class={css({
							fontSize: '16px',
							color: '#4A5568',
							marginTop: '4px'
						})}
					>
						by the same author
					</p>
				</div>
				<div
					class={css({
						flex: '1',
						backgroundColor: '#EBF8FF',
						padding: '16px',
						borderRadius: '8px',
						textAlign: 'center',
						position: 'relative',
						display: 'flex',
						flexDirection: 'column',
						alignItems: 'center',
						gap: '8px'
					})}
				>
						<div
							class={css({
								height: '64px',
								position: 'relative',
								display: 'flex',
								alignItems: 'center',
								justifyContent: 'center',
								zIndex: 1
							})}
						>
							<p
								class={css({
									fontSize: '36px',
									fontWeight: 'bold',
									color: '#2B6CB0',
									position: 'relative',
									zIndex: 1
								})}
							>
								{(result.confidence * 100).toFixed(2)}%
							</p>
						</div>
					<p
						class={css({
							fontSize: '16px',
							color: '#4A5568',
							marginTop: '4px'
						})}
					>
						Confidence
					</p>
				</div>
			</div>

			<h3
				class={css({
					fontSize: '20px',
					fontWeight: 'bold',
					marginBottom: '16px',
					color: '#2D3748',
				})}
			>
				Detailed Analysis
			</h3>
			<div
				class={css({
					display: 'grid',
					gap: '16px',
					gridTemplateColumns: {
						base: '1fr',
						md: 'repeat(2, 1fr)',
					},
				})}
			>
				{result.detailed_analysis.map((detail) => (
					<div
						key={detail.aspect}
						class={css({
							backgroundColor: '#F7FAFC',
							padding: '16px',
							borderRadius: '8px',
							border: '1px solid #E2E8F0',
						})}
					>
						<h4
							class={css({
								fontSize: '16px',
								fontWeight: 'bold',
								marginBottom: '8px',
								color: '#4A5568',
							})}
						>
							{detail.aspect}
						</h4>
						<p
							class={css({
								fontSize: '14px',
								color: '#4A5568',
								marginBottom: '8px',
							})}
						>
							Difference: {(detail.difference * 100).toFixed(2)}%
						</p>
						<p
							class={css({
								fontSize: '14px',
								color: '#718096',
								lineHeight: '1.5',
							})}
						>
							{detail.explanation}
						</p>
					</div>
				))}
			</div>
		</div>
	)
}
